
use indexmap::IndexMap;

use serde::{Serialize, Deserialize};

// the values need to be normalized
// to the correct (x, y) values
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Displays {
	// need to use IndexMap to use rayon
	pub inner: IndexMap<DisplayKind, Display>
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayKind {
	LeftMfcd,
	RightMfcd,
	CenterMfd
}

impl DisplayKind {
	pub fn as_u8(&self) -> u8 {
		match self {
			Self::LeftMfcd => 5,
			Self::RightMfcd => 10,
			Self::CenterMfd => 15
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