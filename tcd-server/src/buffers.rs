// todo benchmark if this is faster than allocating

use std::{mem, slice};
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ops::{Deref, DerefMut};

// use bytes::
use simple_bytes::{BytesOwned, BytesRead, BytesSeek};

static BUFFERS: BufferPool = BufferPool::new(150);


pub struct BufferPool {
	inner: OnceLock<Buffers>,
	cap: usize
}

impl BufferPool {
	fn get(&self) -> &Buffers {
		self.inner.get_or_init(|| Buffers::new(self.cap))
	}

	pub const fn new(cap: usize) -> Self {
		Self {
			inner: OnceLock::new(),
			cap
		}
	}

	/// if the capacity is zero any capacity can be returned
	pub fn take(&self, cap: usize) -> Buffer {
		self.get().take(cap)
	}

	/// if the capacity is zero any capacity can be returned
	///
	/// This might return a buffer which is not of len 0
	pub fn take_raw(&self, cap: usize) -> Buffer {
		self.get().take_raw(cap)
	}
}

#[derive(Debug)]
struct Buffers {
	inner: Mutex<Vec<BytesOwned>>,
	max_cap: usize
}

impl Buffers {
	fn new(max_cap: usize) -> Self {
		Self {
			inner: Mutex::new(vec![]),
			max_cap
		}
	}

	/// the buffer might not be empty
	///
	/// But the cursor will always be at 0
	fn take_raw(&self, cap: usize) -> Buffer {
		let mut bytes = {
			let mut inner = self.inner.lock().unwrap();
			inner.pop()
				.unwrap_or_else(BytesOwned::new)
		};

		bytes.as_mut_vec().reserve_exact(cap);
		bytes.seek(0);

		Buffer {
			inner: self,
			bytes
		}
	}

	/// the buffer will always have a len of 0
	fn take(&self, cap: usize) -> Buffer {
		let mut b = self.take_raw(cap);
		b.resize(0);
		b
	}

	fn return_buffer(&self, mut bytes: BytesOwned) {
		// let cap = bytes.as_mut_vec().capacity();
		// if cap > 620_000 {
		// 	eprintln!("returned buffer with cap {:?}", cap);
		// }

		// return;

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
	///
	/// Do not call this if the buffers are big
	pub fn new() -> Self {
		BUFFERS.take(0)
	}

	#[allow(dead_code)]
	pub fn with_capacity(cap: usize) -> Self {
		BUFFERS.take(cap)
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
			inner: BUFFERS.get(),
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

		let shared_two = shared.clone();

		let bytes = shared.into_bytes();
		assert_eq!(&bytes[4..], nums);

		assert_eq!(shared_two.clone().into_bytes(), bytes);
		drop(shared_two);

		// we expect that the current buffer still exists so we will receive
		// a new buffer
		assert_eq!(Buffer::new().as_mut_vec().capacity(), 0);

		// now make sure that all buffers are dropped
		drop(bytes);

		// we should now receive a buffer that has more capacity
		assert!(Buffer::new().as_mut_vec().capacity() > 0);
	}
}