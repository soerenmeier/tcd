
use std::mem;
use std::sync::Arc;
use std::collections::{HashMap, hash_map};

use tokio::sync::watch;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub struct DisplaySetup {
	inner: Arc<watch::Sender<Option<Displays>>>
}

impl DisplaySetup {
	pub fn new() -> Self {
		Self {
			inner: Arc::new(watch::channel(None).0)
		}
	}

	#[allow(dead_code)]
	pub fn get(&self) -> Option<Displays> {
		self.inner.borrow().clone()
	}

	pub fn set(&self, displays: Option<Displays>) {
		self.inner.send_replace(displays);
	}

	pub fn subscribe(&self) -> DisplaySetupWatcher {
		DisplaySetupWatcher {
			inner: self.inner.subscribe()
		}
	}
}

pub struct DisplaySetupWatcher {
	inner: watch::Receiver<Option<Displays>>
}

impl DisplaySetupWatcher {
	pub fn has_changed(&self) -> bool {
		self.inner.has_changed().unwrap()
	}

	pub fn clone(&mut self) -> Option<Displays> {
		self.inner.borrow_and_update().clone()
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Displays {
	inner: HashMap<DisplayKind, Display>
}

impl Displays {
	pub fn default() -> Self {
		let mut map = HashMap::new();
		map.insert(DisplayKind::LeftMfcd, Display {
			x: 0,
			y: 0,
			width: 640,
			height: 640
		});
		map.insert(DisplayKind::RightMfcd, Display {
			x: 640,
			y: 0,
			height: 640,
			width: 640
		});

		Self { inner: map }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DisplayKind {
	LeftMfcd,
	RightMfcd,
	CenterMfd
}

impl DisplayKind {
	pub fn from_u8(num: u8) -> Option<Self> {
		match num {
			5 => Some(Self::LeftMfcd),
			10 => Some(Self::RightMfcd),
			15 => Some(Self::CenterMfd),
			_ => None
		}
	}
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Display {
	pub x: u32,
	pub y: u32,
	pub width: u32,
	pub height: u32
}

#[derive(Debug, Clone)]
pub struct DisplayFrames {
	inner: HashMap<DisplayKind, Vec<u8>>
}

impl DisplayFrames {
	pub fn new() -> Self {
		Self {
			inner: HashMap::new()
		}
	}

	pub fn get(&self, kind: &DisplayKind) -> Option<&Vec<u8>> {
		self.inner.get(kind)
	}

	pub fn insert(&mut self, kind: DisplayKind, buf: Vec<u8>) {
		self.inner.insert(kind, buf);
	}

	pub fn keys(&self) -> hash_map::Keys<DisplayKind, Vec<u8>> {
		self.inner.keys()
	}

	pub fn remove(&mut self, kind: &DisplayKind) -> Option<Vec<u8>> {
		self.inner.remove(kind)
	}

	pub fn clear_and_get_buffers(&mut self, buffers: &mut Vec<Vec<u8>>) {
		for buf in self.inner.values_mut() {
			buffers.push(mem::take(buf));
		}
		self.inner.clear();
	}
}