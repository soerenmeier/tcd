
use crate::texture_buffer::TextureBuffer;
use crate::displays::{Displays, Display};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
#[cfg(feature = "log-framerate")]
use std::time::Instant;
use std::{thread, fs};
use std::net::TcpStream;
use std::io::{self, Write, Cursor, BufReader, Read};
use std::path::Path;

use image::{RgbaImage, ImageOutputFormat};

use rayon::prelude::*;

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
			let mut displays = None;

			loop {
				let r = connection_loop(&inner, &mut parker, &mut displays);
				match r {
					Ok(_) => break,
					// failed to connect
					// let's wait a bit longer
					Err(Error::Connecting(_)) => {
						thread::sleep(Duration::from_secs(5));
					},
					Err(e) => {
						log(&format!("connection error {:?}\n", e));
						thread::sleep(Duration::from_secs(1));
					},
				}
			}
		});

		Self {
			has_locked: AtomicBool::new(false),
			unparker, inner
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
	Serde(serde_json::Error),
	Image(image::ImageError)
}

const TOTAL_WIDTH: usize = 1920;
const TOTAL_HEIGHT: usize = 1080;
const BYTES_PER_PIXEL: usize = 4;
const LEN: usize = TOTAL_WIDTH * TOTAL_HEIGHT * BYTES_PER_PIXEL;
const EXPECTED_LEN: usize = 640 * 640 * BYTES_PER_PIXEL;

fn copy_display_frame(display: &Display, data: &[u8], to: &mut [u8]) {
	for n_y in 0..display.height {
		let n_y = n_y as usize;
		let m_y = display.y as usize;
		let m_w = display.width as usize;

		let r_y = m_y + n_y;
		let r_x = display.x as usize;// end is display.width

		let data_start = (r_y * TOTAL_WIDTH + r_x) * BYTES_PER_PIXEL;

		let to_start = n_y * m_w * BYTES_PER_PIXEL;
		let bytes_len = m_w * BYTES_PER_PIXEL;
		to[to_start..(to_start + bytes_len)].copy_from_slice(
			&data[data_start..(data_start + bytes_len)]
		);
	}
}

fn log(s: &str) {
	let path = Path::new(r"C:\tcd\logs.txt");
	if !path.is_file() {
		return;
	}

	let mut file = fs::OpenOptions::new()
		.write(true)
		.append(true)
		.open(path)
		.expect("failed to open logs.txt");
	file.write_all(s.as_bytes())
		.expect("failed to write to logs.txt");
}

fn connection_loop(
	inner: &Inner,
	parker: &mut Parker,
	displays: &mut Option<Displays>
) -> Result<(), Error> {
	let stream = TcpStream::connect(ADDR)
		.map_err(Error::Connecting)?;
	let mut reader = BufReader::new(stream);

	// ┌────────────┐
	// |Displays Len│
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
	let mut image_buffers = vec![];
	let mut recv_buffer = Vec::with_capacity(1024);

	loop {
		parker.park();

		// if the virtual Display was dropped stop the thread
		if inner.was_dropped.load(Ordering::SeqCst) {
			return Ok(())
		}

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

		if let Some(len) = len {
			let len = len as usize;
			recv_buffer.resize(len, 0);
			reader.read_exact(&mut recv_buffer[..len])
				.map_err(Error::Transmission)?;

			*displays = serde_json::from_slice(&recv_buffer[..len])
				.map_err(Error::Serde)?;
		}

		// get some data
		let data = inner.buffer.get();
		if data.len() != LEN {
			continue
		}

		let displays = match &displays {
			Some(displays) => displays,
			None => {
				// send 0 displays
				reader.get_mut().write_all(&[0])
					.map_err(Error::Transmission)?;
				continue
			}
		};

		#[cfg(feature = "log-framerate")]
		let start = Instant::now();

		// let's send how many displays we will send
		let displays_len = displays.inner.len() as u8;
		reader.get_mut().write_all(&[displays_len])
			.map_err(Error::Transmission)?;

		if displays_len as usize != image_buffers.len() {
			image_buffers.resize(
				displays_len as usize,
				Vec::with_capacity(EXPECTED_LEN)
			);
		}

		displays.inner.par_iter().zip(&mut image_buffers)
			.for_each(|((kind, display), image_buffer)| {
				let size = display.width * display.height *
					BYTES_PER_PIXEL as u32;
				let mut buffer = vec![0; size as usize];

				// now get the range from data
				copy_display_frame(display, &data, &mut buffer);

				image_buffer.resize(5, 0);
				let mut cursor = Cursor::new(image_buffer);
				cursor.set_position(5);

				// now convert the raw bytes to a jpeg
				let image = RgbaImage::from_vec(
					display.width,
					display.height,
					buffer
				);
				if let Some(image) = image {
					let r = image.write_to(
						&mut cursor,
						ImageOutputFormat::Jpeg(80)
					).map_err(Error::Image);
					if let Err(e) = r {
						log(&format!("image error {:?}", e));
						return;
					}
				}

				let image_buffer = cursor.into_inner();

				let len = image_buffer.len() - 5;
				image_buffer[0] = kind.as_u8();
				let be_bytes = (len as u32).to_be_bytes();
				image_buffer[1..5].copy_from_slice(&be_bytes);
			});

		for image_buffer in &image_buffers {
			// now send the data
			reader.get_mut().write_all(&image_buffer)
				.map_err(Error::Transmission)?;
		}

		#[cfg(feature = "log-framerate")]
		{
			let took = start.elapsed();
			log(&format!("display loop took: {}ms\n", took.as_millis()));
		}
	}
}