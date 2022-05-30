
use crate::displays::{
	DisplaySetup, DisplaySetupWatcher, DisplayFrames, DisplayKind
};

use std::{io, mem};
use std::sync::Arc;
use std::collections::HashSet;

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{BufReader, AsyncReadExt, AsyncWriteExt};
use tokio::sync::watch;
use tokio::task::JoinHandle;

const ADDR: &str = "127.0.0.1:5476";

macro_rules! io_other {
	($err:expr) => {
		io::Error::new(io::ErrorKind::Other, $err)
	}
}



#[derive(Debug, Clone)]
pub struct VirtualDisplay {
	inner: watch::Receiver<DisplayFrames>
}

impl VirtualDisplay {
	pub fn new(display_setup: DisplaySetup) -> (Self, JoinHandle<()>) {
		let (tx, rx) = watch::channel(DisplayFrames::new());
		let this = Self {
			inner: rx
		};

		let handle = tokio::spawn(async move {
			listener_task(tx, display_setup).await
		});

		(this, handle)
	}

	pub async fn recv(
		&mut self,
		subscribed: &HashSet<DisplayKind>
	) -> DisplayFrames {
		self.inner.changed().await.expect("virtual display task failed");
		let data = self.inner.borrow();
		let mut n_data = DisplayFrames::new();
		for kind in subscribed {
			if let Some(image) = data.get(kind) {
				n_data.insert(*kind, image.clone());
			}
		}

		n_data
	}
}

async fn listener_task(
	tx: watch::Sender<DisplayFrames>,
	display_setup: DisplaySetup
) {
	let listener = TcpListener::bind(ADDR).await
		.expect("failed to bind listener");

	eprintln!("virtual display driver listening on {}", ADDR);

	let tx = Arc::new(tx);

	loop {

		let stream = listener.accept().await;
		match stream {
			Ok((stream, addr)) => {
				let tx = tx.clone();
				let display_setup = display_setup.subscribe();
				tokio::spawn(async move {
					eprintln!("virtual display connected from {}", addr);

					let r = handle_stream(stream, tx, display_setup).await;
					if let Err(e) = r {
						eprintln!("stream error {:?}", e);
					}
				});
			},
			Err(e) => {
				eprintln!("failed to accept virtual display connection {:?}", e);
			}
		}

	}
}

async fn handle_stream(
	stream: TcpStream,
	tx: Arc<watch::Sender<DisplayFrames>>,
	mut display_setup: DisplaySetupWatcher
) -> io::Result<()> {
	let mut reader = BufReader::new(stream);

	let mut sent_displays = false;
	let mut buffers = vec![];
	let mut read_buf = vec![];
	let mut prev_data = tx.borrow().clone();

	loop {
		let has_changed = display_setup.has_changed() || !sent_displays;
		if has_changed {
			let maybe_displays = display_setup.clone();
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
		// reserved 5bytes
		let displays_len = reader.read_u8().await?;

		if displays_len == 0 {
			continue
		}

		prev_data.clear_and_get_buffers(&mut buffers);

		for _ in 0..displays_len {
			let kind = reader.read_u8().await?;
			let kind = DisplayKind::from_u8(kind)
				.ok_or_else(|| io_other!(format!("invalid kind {}", kind)))?;
			let len = reader.read_u32().await? as usize;

			read_buf.resize(len, 0);

			reader.read_exact(&mut read_buf[..len]).await?;

			// to save on allocations
			let image = match buffers.pop() {
				Some(buf) => mem::replace(&mut read_buf, buf),
				None => read_buf.clone()
			};
			prev_data.insert(kind, image);
		}

		buffers.clear();
		prev_data = tx.send_replace(prev_data);
	}
}
