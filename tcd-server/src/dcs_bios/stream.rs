
use std::io;

use tokio::net::TcpStream;
use tokio::io::{BufReader, AsyncReadExt, AsyncWriteExt};

enum RecvState {
	Sync,
	Data
}

const SYNC_BYTES: [u8; 4] = [0x55; 4];
// we can store all addresses from 0-u16::MAX so our length is u16::MAX + 1
const BUFFER_LEN: usize = u16::MAX as usize + 1;

macro_rules! io_other {
	($err:expr) => {
		io::Error::new(io::ErrorKind::Other, $err)
	}
}

pub(super) struct Stream {
	inner: BufReader<TcpStream>,
	// has a len of u16::MAX
	buffer: Vec<u8>,
	recv_state: RecvState
}

impl Stream {
	pub async fn connect() -> io::Result<Self> {
		let stream = TcpStream::connect("127.0.0.1:7778").await?;

		Ok(Self {
			inner: BufReader::new(stream),
			buffer: vec![0; BUFFER_LEN],
			recv_state: RecvState::Sync
		})
	}

	/// if we read the entire buffer return it else returns None.
	///
	/// If an Err is returned this means the stream is broken.
	///
	/// ## Note
	/// This function is not abort safe
	pub async fn read(&mut self) -> io::Result<Option<&[u8]>> {
		match self.recv_state {
			RecvState::Sync => {
				// we expect 4bytes each with the value 0x55
				let mut buf = [0u8; 4];
				self.inner.read_exact(&mut buf).await?;

				if buf == SYNC_BYTES {
					self.recv_state = RecvState::Data;
					Ok(None)
				} else {
					Err(io_other!("expected sync bytes"))
				}
			},
			RecvState::Data => {
				// lets receive the addr
				let addr = self.inner.read_u16_le().await? as usize;
				let len = self.inner.read_u16_le().await? as usize;
				if addr + len > BUFFER_LEN {
					eprintln!("len {} max: {}", addr + len, u16::MAX);
					return Err(io_other!("invalid addr or length"))
				}

				let buf = &mut self.buffer[addr..][..len];
				self.inner.read_exact(buf).await?;

				if addr + len == BUFFER_LEN {
					self.recv_state = RecvState::Sync;
					// we have received every data
					Ok(Some(&self.buffer))
				} else {
					Ok(None)
				}
			}
		}
	}

	pub async fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
		self.inner.write_all(bytes).await
	}
}