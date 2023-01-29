//! kinds: debug, info, warn, error, framerate, timepassed

use std::path::Path;
use std::fs::{self, File};
use std::io::{self, Write};
use std::collections::HashSet;
use std::sync::{Mutex, LazyLock};
use std::sync::atomic::{AtomicBool, Ordering};


static LOGGER: LazyLock<Logger> = LazyLock::new(Logger::new);

pub fn should_log(kind: &str) -> bool {
	LOGGER.should_log(kind)
}

pub fn log(s: impl Into<String>) {
	LOGGER.log(s);
}

macro_rules! log {
	($kind:expr, $($arg:tt)*) => (
		if $crate::logger::should_log($kind) {
			$crate::logger::log(format!($($arg)*));
		}
	)
}

#[allow(unused_macros)]
macro_rules! debug {
	($($arg:tt)*) => (
		log!("debug", $($arg)*);
	)
}

#[allow(unused_macros)]
macro_rules! info {
	($($arg:tt)*) => (
		log!("info", $($arg)*);
	)
}

#[allow(unused_macros)]
macro_rules! warn {
	($($arg:tt)*) => (
		log!("warn", $($arg)*);
	)
}

#[allow(unused_macros)]
macro_rules! error {
	($($arg:tt)*) => (
		log!("error", $($arg)*);
	)
}


struct Logger {
	// needs to be able to be changed since if we fail to write to the file
	// we disable the logger
	enabled: AtomicBool,
	// if the set is empty all kinds should be logged
	kinds_to_log: HashSet<String>,
	file: Mutex<Option<File>>
}

impl Logger {
	pub fn new() -> Self {
		let kinds_file = Path::new(r"C:\tcd\log.txt");
		let Ok(kinds_file) = fs::read_to_string(kinds_file) else {
			return Self {
				enabled: AtomicBool::new(false),
				kinds_to_log: HashSet::new(),
				file: Mutex::new(None)
			}
		};
		let kinds: HashSet<_> = kinds_file.split(',')
			.map(str::trim)
			.filter(|s| !s.is_empty())
			.map(ToString::to_string)
			.collect();

		Self {
			enabled: AtomicBool::new(true),
			kinds_to_log: kinds,
			file: Mutex::new(None)
		}
	}

	fn should_log(&self, kind: &str) -> bool {
		self.enabled.load(Ordering::Relaxed) && (
			self.kinds_to_log.is_empty() || self.kinds_to_log.contains(kind)
		)
	}

	fn log(&self, s: impl Into<String>) {
		let mut s = s.into();
		if !s.ends_with('\n') {
			s.push('\n');
		}

		if let Err(_) = self.try_log(&s) {
			// failed to write to the file so let's disabled the logger
			self.enabled.store(false, Ordering::Relaxed);
			*self.file.lock().unwrap() = None;
		}
	}

	fn try_log(&self, s: &str) -> io::Result<()> {
		let mut file = self.file.lock().unwrap();
		let file = match file.as_mut() {
			Some(f) => f,
			None => {
				let n_file = fs::OpenOptions::new()
					.append(true)
					.create(true)
					.open(r"C:\tcd\tcd.log")?;

				file.insert(n_file)
			}
		};
		
		file.write_all(s.as_bytes())
	}
}