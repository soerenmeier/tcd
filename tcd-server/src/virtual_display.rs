use crate::displays::{
	DisplaySetup, DisplaySetupWatcher, SharedDisplayFrames, DisplayKind,
	FrameReceiver
};
use crate::buffers::Buffer;

use std::io;

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{BufReader, AsyncReadExt, AsyncWriteExt};
use tokio::task::JoinHandle;

use simple_bytes::{Bytes, BytesRead};


const ADDR: &str = "127.0.0.1:5476";

macro_rules! io_other {
	($err:expr) => {
		io::Error::new(io::ErrorKind::Other, $err)
	}
}



#[derive(Debug, Clone)]
pub struct VirtualDisplay {
	inner: SharedDisplayFrames
}

impl VirtualDisplay {
	pub fn new(display_setup: DisplaySetup) -> (Self, JoinHandle<()>) {
		let inner = SharedDisplayFrames::new();
		let this = Self { inner: inner.clone() };

		let handle = tokio::spawn(async move {
			listener_task(inner, display_setup).await
		});

		(this, handle)
	}

	pub fn receiver(&self, kind: &DisplayKind) -> Option<FrameReceiver> {
		self.inner.receiver(kind)
	}
}

async fn listener_task(
	frames: SharedDisplayFrames,
	display_setup: DisplaySetup
) {
	let listener = TcpListener::bind(ADDR).await
		.expect("failed to bind listener");

	eprintln!("virtual display driver listening on {}", ADDR);

	loop {

		let stream = listener.accept().await;
		match stream {
			Ok((stream, addr)) => {
				let frames = frames.clone();
				let display_setup = display_setup.subscribe();
				tokio::spawn(async move {
					eprintln!("virtual display connected from {addr}");

					let r = handle_stream(stream, frames, display_setup).await;
					if let Err(e) = r {
						eprintln!("stream error {e:?}");
					}
				});
			},
			Err(e) => {
				eprintln!("failed to accept virtual display connection {e:?}");
			}
		}

	}
}

async fn handle_stream(
	stream: TcpStream,
	shared_frames: SharedDisplayFrames,
	mut display_setup: DisplaySetupWatcher
) -> io::Result<()> {
	let mut reader = BufReader::new(stream);
	// we keep a local copy of the frames to make sure that the stream
	// always receives the correct resolution buffers
	let mut frames = shared_frames.clone_inner();

	let mut sent_displays = false;
	let mut read_buf = vec![];

	loop {
		let has_changed = display_setup.has_changed() || !sent_displays;
		if has_changed {
			let maybe_displays = display_setup.clone();
			if let Some(displays) = &maybe_displays {
				frames = shared_frames.update_from_displays(displays);
			}

			let v = serde_json::to_vec(&maybe_displays).unwrap();
			reader.write_u32(v.len() as u32).await?;
			reader.write_all(&v).await?;

			sent_displays = true;
		}

		// ┌────────────┐
		// │Displays Len│
		// ├────────────┤
		// │     8      │
		// └────────────┘
		//
		// ┌────┬───┬──────┐
		// │Kind│Len│ Data │
		// ├────┼───┼──────┤
		// │ 8  │32 │$Len*8│
		// └────┴───┴──────┘
		//
		// Data Packet (a data packet consists of many nal)
		// ┌─────┬──────┐
		// │ Len │ Data │
		// ├─────┼──────┤
		// │ 32  │$Len*8│
		// └─────┴──────┘
		let displays_len = reader.read_u8().await?;

		if displays_len == 0 {
			continue
		}

		for _ in 0..displays_len {
			let kind = reader.read_u8().await?;
			let kind = DisplayKind::from_u8(kind)
				.ok_or_else(|| io_other!(format!("invalid kind {}", kind)))?;
			let len = reader.read_u32().await? as usize;

			if len == 0 {
				continue
			}

			read_buf.resize(len, 0);

			reader.read_exact(&mut read_buf[..len]).await?;

			let mut bytes = Bytes::from(read_buf.as_slice());

			let mut nal_count = 0;
			while !bytes.remaining().is_empty() {
				let len = bytes.try_read_u32()
					.map_err(|e| io_other!(format!("proto error {e:?}")))?;
				let nal = bytes.try_read(len as usize)
					.map_err(|e| io_other!(format!("proto error {e:?}")))?;

				let mut buffer = Buffer::new();
				buffer.as_mut_vec().extend_from_slice(nal);
				frames.send_buffer(&kind, buffer.into_shared());

				tracing::info!("received buffer");

				nal_count += 1;
			}

			// max was 3
			// eprintln!("received {kind:?} {nal_count:?} nals with a len of {len:?}");
		}
	}
}
