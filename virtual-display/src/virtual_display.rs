use crate::texture_buffer::TextureBuffer;
use crate::displays::Displays;
use crate::yuv::bgra_to_yuv420;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::time::Instant;
use std::thread;
use std::net::TcpStream;
use std::io::{self, Write, BufReader, Read};

// use rayon::prelude::*;

use simple_bytes::{BytesOwned, BytesRead, BytesSeek, BytesWrite};

use crossbeam_utils::sync::{Parker, Unparker};



const ADDR: &str = "127.0.0.1:5476";

// use crossbeam::channel::Receiver;

struct Inner {
	was_dropped: AtomicBool,
	buffer: TextureBuffer
}

pub struct VirtualDisplay {
	// tracks if the swap chain as locked the buffer
	has_locked: AtomicBool,
	unparker: Unparker,
	last_send: Mutex<Instant>,
	inner: Arc<Inner>
}

impl VirtualDisplay {
	pub fn new() -> Self {
		let inner = Arc::new(Inner {
			was_dropped: AtomicBool::new(false),
			buffer: TextureBuffer::new()
		});
		let n_inner = inner.clone();

		let parker = Parker::new();
		let unparker = parker.unparker().clone();

		thread::spawn(move || {
			let inner = n_inner;
			let mut parker = parker;

			loop {
				let r = connection_loop(&inner, &mut parker);
				match r {
					Ok(_) => break,
					// failed to connect
					// let's wait a bit longer
					Err(Error::Connecting(_)) => {
						thread::sleep(Duration::from_secs(5));
					},
					Err(e) => {
						error!("connection error {e:?}");
						thread::sleep(Duration::from_secs(1));
					},
				}
			}
		});

		Self {
			has_locked: AtomicBool::new(false),
			unparker, inner,
			last_send: Mutex::new(Instant::now())
		}
	}

	pub fn has_locked(&self) -> bool {
		self.has_locked.load(Ordering::SeqCst)
	}

	pub fn set_locked(&self, locked: bool) {
		self.has_locked.store(locked, Ordering::SeqCst)
	}

	pub fn buffer(&self) -> &TextureBuffer {
		&self.inner.buffer
	}

	pub fn time_since_last_send(&self) -> Duration {
		self.last_send.lock().unwrap()
			.elapsed()
	}

	pub fn update_last_send(&self) {
		let mut timer = self.last_send.lock().unwrap();
		*timer = Instant::now();
	}

	pub fn notify(&self) {
		self.unparker.unpark();
	}
}

impl Drop for VirtualDisplay {
	fn drop(&mut self) {
		self.inner.was_dropped.store(true, Ordering::SeqCst);
		self.unparker.unpark();
	}
}

#[derive(Debug)]
enum Error {
	Connecting(io::Error),
	Transmission(io::Error),
	Serde(serde_json::Error)
}

const TOTAL_WIDTH: usize = 1920;
const TOTAL_HEIGHT: usize = 1080;
const BYTES_PER_PIXEL: usize = 4;
const LEN: usize = TOTAL_WIDTH * TOTAL_HEIGHT * BYTES_PER_PIXEL;
// ┌────┬───┬──────┐
// │Kind│Len│ Data │
// ├────┼───┼──────┤
// │ 8  │32 │$Len*8│
// └────┴───┴──────┘
const DISPLAY_BUFFER_OFFSET: usize = 1 + 4;

const FRAME_RATE: Duration = Duration::from_millis(1000 / 30);


fn connection_loop(
	inner: &Inner,
	parker: &mut Parker
) -> Result<(), Error> {
	let stream = TcpStream::connect(ADDR)
		.map_err(Error::Connecting)?;
	let mut reader = BufReader::new(stream);

	let mut displays: Option<Displays> = None;

	// Packet one
	// ┌────────────┐
	// |Displays Len│
	// ├────────────┤
	// │     8      │
	// └────────────┘
	// 
	// Display Packets
	// ┌────┬───┬──────┐
	// │Kind│Len│ Data │
	// ├────┼───┼──────┤
	// │ 8  │32 │$Len*8│
	// └────┴───┴──────┘
	let mut display_buffers = vec![];
	let mut recv_buffer = Vec::with_capacity(1024);

	let mut last_frame_sent = Instant::now();

	loop {
		parker.park();

		// if the virtual Display was dropped stop the thread
		if inner.was_dropped.load(Ordering::SeqCst) {
			return Ok(())
		}

		let n_displays = maybe_read_new_displays(
			&mut reader,
			&mut recv_buffer
		)?;

		if let Some(displs) = n_displays {
			info!("received displays {displs:?}");

			display_buffers.clear();
			for display in displs.inner.values() {
				let yuv_len = (display.width * display.height * 3) / 2;
				let size = DISPLAY_BUFFER_OFFSET + yuv_len as usize;
				display_buffers.push(BytesOwned::with_capacity(size));
			}

			displays = Some(displs);
		}

		// get some data
		let data = inner.buffer.get();
		if data.len() != LEN {
			continue
		}

		if last_frame_sent.elapsed() < FRAME_RATE {
			continue
		}

		let Some(displays) = &displays else {
			// send 0 displays
			reader.get_mut().write_all(&[0])
				.map_err(Error::Transmission)?;
			continue
		};

		let start = Instant::now();

		// let's send how many displays we will send
		let displays_len = displays.inner.len() as u8;
		reader.get_mut().write_all(&[displays_len])
			.map_err(Error::Transmission)?;

		// todo should we remove the par iter??
		displays.inner.iter()
		// displays.inner.par_iter()
			.zip(&mut display_buffers)
			.for_each(|((kind, display), display_buffer)| {
				let width = display.width;
				let height = display.height;



				let yuv_len = (display.width * display.height * 3) / 2;
				display_buffer.resize(DISPLAY_BUFFER_OFFSET + yuv_len as usize);

				display_buffer.seek(0);
				display_buffer.write_u8(kind.as_u8());
				// write a temp len
				display_buffer.write_u32(yuv_len);

				// convert bgra to yuv
				bgra_to_yuv420(
					&data,
					TOTAL_WIDTH as u32, TOTAL_HEIGHT as u32,
					display.x, display.y,
					width, height,
					display_buffer.remaining_mut()
				);
			});

		for display_buffer in &display_buffers {
			// now send the data
			reader.get_mut().write_all(display_buffer.as_slice())
				.map_err(Error::Transmission)?;
		}

		let took = start.elapsed();
		let took_last = last_frame_sent.elapsed();
		last_frame_sent = Instant::now();
		log!(
			"framerate",
			"display loop took: {}ms, since last frame {}ms",
			took.as_millis(),
			took_last.as_millis()
		);
	}
}

fn maybe_read_new_displays(
	reader: &mut BufReader<TcpStream>,
	recv_buffer: &mut Vec<u8>
) -> Result<Option<Displays>, Error> {
	// check if we received some data
	reader.get_mut().set_nonblocking(true)
		.map_err(Error::Transmission)?;
	let len = {
		let mut bytes = [0u8; 4];
		match reader.read_exact(&mut bytes) {
			Ok(_) => Some(u32::from_be_bytes(bytes)),
			Err(e) if e.kind() == io::ErrorKind::WouldBlock => None,
			Err(e) => return Err(Error::Transmission(e))
		}
	};
	reader.get_mut().set_nonblocking(false)
		.map_err(Error::Transmission)?;

	let Some(len) = len else {
		return Ok(None)
	};

	let len = len as usize;
	recv_buffer.resize(len, 0);
	reader.read_exact(&mut recv_buffer[..len])
		.map_err(Error::Transmission)?;

	serde_json::from_slice(&recv_buffer[..len])
		.map(Some)
		.map_err(Error::Serde)
}