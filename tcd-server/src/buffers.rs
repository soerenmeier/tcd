use std::{mem, slice};
use std::sync::{Arc, Mutex, LazyLock};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ops::{Deref, DerefMut};

// use bytes::
use simple_bytes::{BytesOwned, BytesRead};

const BUFFERS_CAP: usize = 30;
static BUFFERS: LazyLock<Buffers> = LazyLock::new(|| Buffers::new(BUFFERS_CAP));


#[derive(Debug)]
struct Buffers {
	inner: Mutex<Vec<BytesOwned>>,
	max_cap: usize
}

impl Buffers {
	pub fn new(max_cap: usize) -> Self {
		Self {
			inner: Mutex::new(vec![]),
			max_cap
		}
	}

	pub fn take(&self) -> Buffer {
		let mut bytes = {
			let mut inner = self.inner.lock().unwrap();
			inner.pop()
				.unwrap_or_else(BytesOwned::new)
		};

		bytes.resize(0);

		Buffer {
			inner: self,
			bytes
		}
	}

	fn return_buffer(&self, mut bytes: BytesOwned) {
		if bytes.as_mut_vec().capacity() == 0 {
			return
		}

		let mut inner = self.inner.lock().unwrap();
		if inner.len() < self.max_cap {
			inner.push(bytes);
		} else {
			tracing::info!("already enough buffers");
		}
	}
}

#[derive(Debug)]
pub struct Buffer<'a> {
	inner: &'a Buffers,
	bytes: BytesOwned
}

impl Buffer<'static> {
	/// Returns a buffer which is of len 0
	pub fn new() -> Self {
		BUFFERS.take()
	}

	pub fn into_shared(self) -> SharedBuffer {
		SharedBuffer(Arc::new(self))
	}
}

impl Drop for Buffer<'_> {
	fn drop(&mut self) {
		let bytes = mem::replace(&mut self.bytes, BytesOwned::new());
		self.inner.return_buffer(bytes);
	}
}

impl Deref for Buffer<'_> {
	type Target = BytesOwned;

	fn deref(&self) -> &BytesOwned {
		&self.bytes
	}
}

impl DerefMut for Buffer<'_> {
	fn deref_mut(&mut self) -> &mut BytesOwned {
		&mut self.bytes
	}
}

#[derive(Debug, Clone)]
pub struct SharedBuffer(Arc<Buffer<'static>>);

impl SharedBuffer {
	// Creates an empty Buffer
	pub fn new() -> Self {
		Self(Arc::new(Buffer {
			inner: &BUFFERS,
			bytes: BytesOwned::new()
		}))
	}

	pub fn into_bytes(self) -> bytes::Bytes {
		let slice = self.0.bytes.as_slice();
		let ptr = slice.as_ptr() as *const _;
		let len = slice.len();

		let data = AtomicPtr::new(Arc::into_raw(self.0) as *mut _);

		unsafe {
			bytes::Bytes::with_vtable(ptr, len, data, &SHARED_BUFFER_VTABLE)
		}
	}
}

impl Deref for SharedBuffer {
	type Target = BytesOwned;

	fn deref(&self) -> &BytesOwned {
		&self.0.bytes
	}
}


const SHARED_BUFFER_VTABLE: bytes::Vtable = bytes::Vtable {
	clone: vtable_clone,
	to_vec: vtable_to_vec,
	drop: vtable_drop
};

// unsafe fn buffer_from_ptr(data: *const ()) -> Arc<Buffer<'static>> {
// 	unsafe {
// 		let data = data as *const _;
// 		Arc::increment_strong_count(data);
// 		Arc::from_raw(data)
// 	}
// }

unsafe fn vtable_clone(
	data: &AtomicPtr<()>,
	ptr: *const u8,
	len: usize
) -> bytes::Bytes {
	let data = data.load(Ordering::Relaxed) as *const Buffer<'static>;
	// increment strong count
	Arc::increment_strong_count(data);

	let data = AtomicPtr::new(data as *mut _);

	bytes::Bytes::with_vtable(ptr, len, data, &SHARED_BUFFER_VTABLE)
}

unsafe fn vtable_to_vec(
	_data: &AtomicPtr<()>,
	ptr: *const u8,
	len: usize
) -> Vec<u8> {
	slice::from_raw_parts(ptr, len).to_vec()
}

unsafe fn vtable_drop(
	data: &mut AtomicPtr<()>,
	_ptr: *const u8,
	_len: usize
) {
	let buffer = Arc::from_raw(
		(*data.get_mut()) as *const Buffer<'static>
	);

	drop(buffer);
}


#[cfg(test)]
mod tests {
	use super::*;

	use simple_bytes::BytesWrite;

	#[test]
	fn test_into_bytes() {
		let mut b = Buffer::new();
		b.write_u32(20);
		let nums: Vec<u8> = (0..255).collect();
		b.write(&nums);

		let shared = b.into_shared();
		let bytes = shared.into_bytes();
		assert_eq!(&bytes[4..], nums);
	}
}