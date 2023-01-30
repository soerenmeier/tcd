use std::mem;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};


#[derive(Debug)]
pub struct Latest<T> {
	current: usize,
	inner: Arc<Inner<T>>
}

impl<T: Clone> Latest<T> {
	pub fn new(val: T) -> Self {
		Self {
			current: 0,
			inner: Arc::new(Inner {
				version: AtomicUsize::new(0),
				value: Mutex::new(val)
			})
		}
	}

	pub fn update(&self, val: T) -> T {
		let mut value = self.inner.value.lock().unwrap();
		let prev = mem::replace(&mut *value, val);
		// update version
		self.inner.version.fetch_add(1, Ordering::Relaxed);

		prev
	}

	/// Returns the latest value if there is a new avaiable
	pub fn latest(&mut self) -> Option<T> {
		let version = self.inner.version.load(Ordering::Relaxed);
		if version > self.current {
			self.current = version;
			Some(self.inner.value.lock().unwrap().clone())
		} else {
			None
		}
	}
}

impl<T> Clone for Latest<T> {
	fn clone(&self) -> Self {
		Self {
			current: self.current,
			inner: self.inner.clone()
		}
	}
}

#[derive(Debug)]
struct Inner<T> {
	version: AtomicUsize,
	value: Mutex<T>
}