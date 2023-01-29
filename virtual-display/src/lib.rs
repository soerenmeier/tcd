#![feature(once_cell)]

#[macro_use]
mod logger;
mod texture_buffer;
mod displays;
mod virtual_display;
use virtual_display::VirtualDisplay;
mod yuv;

use std::ffi::{c_void, c_char, CStr};
use std::sync::Arc;
use std::time::Duration;

// ## Note
// The pointer is *const Mutex<VirtualDisplay> comming from an arc

type Data = Arc<VirtualDisplay>;

fn data_from_ptr(data: *const c_void) -> Arc<VirtualDisplay> {
	unsafe {
		let data = data as *const _;
		Arc::increment_strong_count(data);
		Arc::from_raw(data)
	}
}

#[no_mangle]
pub extern "C" fn VdShouldLog(kind: *const c_char) -> bool {
	let s = unsafe { CStr::from_ptr(kind) };
	logger::should_log(s.to_str().unwrap())
}

#[no_mangle]
pub extern "C" fn VdLog(s: *const c_char) {
	let s = unsafe { CStr::from_ptr(s) };
	logger::log(s.to_str().unwrap());
}

#[no_mangle]
pub extern "C" fn VdInitData() -> *const c_void {
	let data = Arc::new(VirtualDisplay::new());
	Arc::into_raw(data) as *const c_void
}

#[no_mangle]
pub extern "C" fn VdDestroyData(data: *const c_void) {
	let data: Data = unsafe {
		Arc::from_raw(data as *const _)
	};
	drop(data);
}

const RESEND_EVERY: Duration = Duration::from_secs(2);

#[no_mangle]
pub extern "C" fn VdShouldSendTexture(
	data: *const c_void,
	has_changed: bool
) -> bool {
	has_changed || data_from_ptr(data).time_since_last_send() > RESEND_EVERY
}

#[no_mangle]
pub extern "C" fn VdCreateTextureBuffer(
	data: *const c_void,
	len: u32
) -> *mut u8 {
	let data = data_from_ptr(data);
	let buffer = data.buffer();

	if data.has_locked() {
		// there was probably an error that VdSendTexture was not called
		// let's unlock
		data.set_locked(false);
		unsafe {
			buffer.unlock();
		}
	}

	let buffer = buffer.get_mut();
	data.set_locked(true);

	if buffer.capacity() != len as usize {
		*buffer = Vec::with_capacity(len as usize);
	}

	buffer.resize(len as usize, 0);
	assert_eq!(buffer.capacity(), len as usize);

	buffer.as_mut_ptr()
}

#[no_mangle]
pub extern "C" fn VdSendTexture(data: *const c_void, ptr: *mut u8, len: u32) {
	let data = data_from_ptr(data);
	assert!(data.has_locked());

	let buffer = data.buffer();

	// validate the buffer
	{
		let buffer = unsafe {
			buffer.unsafe_get_mut()
		};

		assert_eq!(buffer.len(), len as usize);
		assert_eq!(buffer.as_mut_ptr(), ptr);
	}

	// set the last send counter
	data.update_last_send();

	// we now are sure that we have the same data
	// now we can unlock
	data.set_locked(false);
	unsafe {
		buffer.unlock();
	}

	// let's wake the bg thread
	data.notify();
}