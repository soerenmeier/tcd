
use std::cell::UnsafeCell;

use parking_lot::RawMutex;
use parking_lot::lock_api::RawMutex as RawMutexTrait;

// struct TextureBuffer<T> {
// 	active: AtomicBool,
// 	data: [Mutex<T>; 2]
// }

// if a write get's to it
// check which one is active

pub struct TextureBuffer {
	mutex: RawMutex,
	data: UnsafeCell<Vec<u8>>
}

impl TextureBuffer {
	pub fn new() -> Self {
		Self {
			mutex: RawMutex::INIT,
			data: UnsafeCell::new(vec![])
		}
	}

	pub fn get_mut(&self) -> &mut Vec<u8> {
		self.mutex.lock();
		unsafe {
			&mut *self.data.get()
		}
	}

	/// You need to own the lock and not already have a reference to the
	/// Vec
	pub unsafe fn unsafe_get_mut(&self) -> &mut Vec<u8> {
		&mut *self.data.get()
	}

	/// This is safe if the mutable reference was dropped but was
	/// requested.
	pub unsafe fn unlock(&self) {
		self.mutex.unlock();
	}

	pub fn get(&self) -> Vec<u8> {
		self.mutex.lock();

		let v = unsafe {
			(*self.data.get()).clone()
		};

		unsafe {
			self.mutex.unlock();
		}

		v
	}
}

unsafe impl Send for TextureBuffer {}
unsafe impl Sync for TextureBuffer {}