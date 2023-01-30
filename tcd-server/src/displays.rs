use crate::buffers::SharedBuffer;

use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet, hash_map::Entry};

use tokio::sync::{watch, broadcast};

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
pub struct SharedDisplayFrames {
	inner: Arc<Mutex<DisplayFrames>>
}

impl SharedDisplayFrames {
	pub fn new() -> Self {
		Self {
			inner: Arc::new(Mutex::new(DisplayFrames::new()))
		}
	}

	pub fn clone_inner(&self) -> DisplayFrames {
		self.inner.lock().unwrap().clone()
	}

	pub fn update_from_displays(&self, displays: &Displays) -> DisplayFrames {
		let mut inner = self.inner.lock().unwrap();
		inner.update_from_displays(displays);
		inner.clone()
	}

	pub fn receiver(&self, kind: &DisplayKind) -> Option<FrameReceiver> {
		self.inner.lock().unwrap()
			.receiver(kind)
	}
}

// we should receive a yuv maximaly 30 times per second
const CHANNEL_SIZE: usize = 1;

#[derive(Debug, Clone)]
pub struct DisplayFrames {
	inner: HashMap<DisplayKind, (Display, broadcast::Sender<SharedBuffer>)>
}

impl DisplayFrames {
	fn new() -> Self {
		Self {
			inner: HashMap::new()
		}
	}

	fn update_from_displays(&mut self, displays: &Displays) {
		let mut remove_kinds: HashSet<_> = self.inner.keys().cloned().collect();

		for (kind, display) in &displays.inner {
			remove_kinds.remove(kind);

			match self.inner.entry(*kind) {
				Entry::Occupied(mut occ) => {
					// check if width or height changed
					let (d, _) = occ.get();
					if d.width != display.width || d.height != display.height {
						let (tx, _) = broadcast::channel(CHANNEL_SIZE);
						occ.insert((display.clone(), tx));
					}
				},
				Entry::Vacant(vac) => {
					let (tx, _) = broadcast::channel(CHANNEL_SIZE);
					vac.insert((display.clone(), tx));
				}
			}
		}

		for kind in remove_kinds {
			self.inner.remove(&kind);
		}
	}

	/// if the kind does not exists nothing happens
	pub fn send_buffer(&self, kind: &DisplayKind, buffer: SharedBuffer) {
		if let Some((_, sender)) = self.inner.get(kind) {
			// ignore if we could send the buffer or not
			let _ = sender.send(buffer);
		}
	}

	pub fn receiver(&self, kind: &DisplayKind) -> Option<FrameReceiver> {
		self.inner.get(kind)
			.map(|(display, tx)| FrameReceiver {
				display: display.clone(),
				rx: tx.subscribe()
			})
	}
}

#[derive(Debug)]
pub struct FrameReceiver {
	#[allow(dead_code)]
	display: Display,
	rx: broadcast::Receiver<SharedBuffer>
}

impl FrameReceiver {
	pub fn display(&self) -> &Display {
		&self.display
	}

	/// Returns the buffer an a lagged count
	pub async fn recv(&mut self) -> Option<(SharedBuffer, u64)> {
		let mut lagged = 0;

		loop {
			match self.rx.recv().await {
				Ok(b) => return Some((b, lagged)),
				Err(broadcast::error::RecvError::Lagged(l)) => {
					lagged += l;
				},
				Err(broadcast::error::RecvError::Closed) => return None
			}
		}
	}
}