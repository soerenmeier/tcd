
use super::controls::{
	ControlDefs, ControlDef, InputDef, InputDefKind, OutputDef, OutputDefKind,
	Outputs, Output, ControlOutputs
};

use std::{env, io};
use std::path::{Path, PathBuf};
use std::collections::{hash_map, HashMap};
use std::sync::{Arc, Mutex, MutexGuard};

use tokio::fs;

use serde::{Serialize, Deserialize};

fn control_ref() -> PathBuf {
	let appdata = env::var("APPDATA").expect("appdata not found");
	PathBuf::from(appdata).join(r"DCS-BIOS\control-reference-json")
}

const AIRCRAFTS: &[&str] = &[
	"AV8BNA",
	"Christen Eagle II",
	"F-14B",
	"F-16C_50",
	"FA-18C_hornet",
	"M-2000C",
	"P-51D"
];

#[derive(Debug, Clone)]
pub struct ControlDefinitions {
	inner: Arc<Mutex<InnerControlDefinitions>>
}

impl ControlDefinitions {
	pub async fn new() -> Result<Self, Error> {
		Ok(Self {
			inner: Arc::new(Mutex::new(InnerControlDefinitions::new().await?))
		})
	}

	pub(super) fn lock(&self) -> MutexGuard<InnerControlDefinitions> {
		self.inner.lock().unwrap()
	}

	// pub fn control_outputs(&self, name: &str, buffer: &[u8]) -> Outputs {
	// 	let defs = self.inner.lock().unwrap();
	// 	defs.control_outputs(name, buffer)
	// 		.unwrap_or_else(Outputs::new)
	// }

	// // returns true if the aircraft could be loaded
	// pub fn load_aircraft(&self, name: &str) -> bool {
	// 	let defs = self.inner.lock().unwrap();
	// 	defs.load_aircraft(name)
	// }
}

#[derive(Debug, Clone)]
pub(super) struct InnerControlDefinitions {
	#[allow(dead_code)]
	metadata: ControlDefs,
	raw_metadata: RawControls,
	aircraft: Option<String>,
	aircrafts: HashMap<String, AicraftControlDefs>
}

impl InnerControlDefinitions {
	pub async fn new() -> Result<Self, Error> {
		let control_ref = control_ref();
		let start = File::new(control_ref.join("MetadataStart.json")).await
			.map_err(|_| Error::FailedToOpenMetadata)?;
		let end = File::new(control_ref.join("MetadataEnd.json")).await
			.map_err(|_| Error::FailedToOpenMetadata)?;
		let common = File::new(control_ref.join("CommonData.json")).await
			.map_err(|_| Error::FailedToOpenMetadata)?;

		let mut metadata = ControlDefs::new();
		let mut raw_metadata = RawControls::new();

		start.insert_defs(&mut raw_metadata, &mut metadata);
		end.insert_defs(&mut raw_metadata, &mut metadata);
		common.insert_defs(&mut raw_metadata, &mut metadata);

		let mut aircrafts = HashMap::new();

		for name in AIRCRAFTS {
			let path = control_ref.join(format!("{}.json", name));
			let file = File::new(&path).await;
			let file = match file {
				Ok(f) => f,
				Err(_) => {
					eprintln!("file not found {:?}", path);
					continue
				}
			};

			let mut aircraft = AicraftControlDefs::new();

			file.insert_defs(&mut aircraft.raw_defs, &mut aircraft.defs);

			aircrafts.insert(name.to_string(), aircraft);
		}

		Ok(Self {
			metadata, raw_metadata, aircrafts,
			aircraft: None
		})
	}

	pub fn control_outputs(
		&self,
		name: &str,
		buffer: &[u8]
	) -> Outputs {
		self.raw_metadata.get(name)
			.or_else(|| {
				self.aircraft.as_ref()
					.and_then(|a| self.aircrafts.get(a))
					.and_then(|a| a.raw_defs.get(name))
			})
			.map(|def| def.outputs(buffer))
			.unwrap_or_else(Outputs::new)
	}

	// returns true if the aircraft could be changed
	// else the aircraft is set to None
	pub fn load_aircraft(&mut self, name: &str) -> bool {
		if self.aircrafts.contains_key(name) {
			self.aircraft = Some(name.to_string());
			true
		} else {
			self.aircraft = None;
			false
		}
	}

	pub fn all_outputs(&self, buffer: &[u8]) -> ControlOutputs {
		let mut outputs = ControlOutputs::new();

		// metadata
		for (name, def) in self.raw_metadata.iter() {
			outputs.insert(name.clone(), def.outputs(buffer));
		}

		// aircraft
		let defs = self.aircraft.as_ref().and_then(|a| self.aircrafts.get(a));
		if let Some(defs) = defs {
			for (name, def) in defs.raw_defs.iter() {
				outputs.insert(name.clone(), def.outputs(buffer));
			}
		}

		outputs
	}
}

#[derive(Debug, Clone)]
struct AicraftControlDefs {
	defs: ControlDefs,
	raw_defs: RawControls
}

impl AicraftControlDefs {
	pub fn new() -> Self {
		Self {
			defs: ControlDefs::new(),
			raw_defs: RawControls::new()
		}
	}
}

#[derive(Debug)]
pub enum Error {
	FailedToOpenMetadata
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
struct File {
	// first category, second name
	inner: HashMap<String, HashMap<String, RawControl>>
}

impl File {
	pub async fn new(path: impl AsRef<Path>) -> io::Result<Self> {
		let s = fs::read_to_string(path).await?;

		serde_json::from_str(&s)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))
	}

	pub fn insert_defs(self, raw: &mut RawControls, defs: &mut ControlDefs) {
		let controls = self.inner.into_iter()
			.flat_map(|(_, controls)| controls.into_iter())
			.map(|(name, val)| (name, val));

		for (name, control) in controls {
			defs.insert(name.clone(), control.to_def());
			raw.insert(name, control);
		}
	}
}

#[derive(Debug, Clone)]
struct RawControls {
	inner: HashMap<String, RawControl>
}

impl RawControls {
	pub fn new() -> Self {
		Self {
			inner: HashMap::new()
		}
	}

	pub fn insert(&mut self, name: String, def: RawControl) {
		self.inner.insert(name, def);
	}

	pub fn get(&self, name: &str) -> Option<&RawControl> {
		self.inner.get(name)
	}

	pub fn iter(&self) -> hash_map::Iter<String, RawControl> {
		self.inner.iter()
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawControl {
	pub category: String,
	#[serde(rename = "identifier")]
	pub name: String,
	#[serde(rename = "control_type")]
	pub kind: String,
	#[serde(default = "String::new")]
	pub description: String,
	pub momentary_positions: Option<String>,
	pub physical_variant: Option<String>,
	pub inputs: Vec<RawInput>,
	pub outputs: Vec<RawOutput>
}

impl RawControl {
	pub fn to_def(&self) -> ControlDef {
		ControlDef {
			category: self.category.clone(),
			kind: self.kind.clone(),
			description: self.description.clone(),
			inputs: self.inputs.iter().map(RawInput::to_def).collect(),
			outputs: self.outputs.iter().map(RawOutput::to_def).collect()
		}
	}

	pub fn outputs(&self, buffer: &[u8]) -> Outputs {
		self.outputs.iter()
			.filter_map(|o| o.read(buffer))
			.collect::<Vec<_>>()
			.into()
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RawInputKind {
	FixedStep,
	SetState,
	VariableStep,
	Action
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawInput {
	pub description: String,
	#[serde(rename = "interface")]
	pub kind: RawInputKind,
	pub max_value: Option<u16>,
	pub suggested_step: Option<u16>,
	pub argument: Option<String>
}

impl RawInput {
	pub fn to_def(&self) -> InputDef {
		InputDef {
			description: self.description.clone(),
			kind: match self.kind {
				RawInputKind::FixedStep => InputDefKind::FixedStep,
				RawInputKind::SetState => InputDefKind::SetState {
					max_value: self.max_value.unwrap() as usize
				},
				RawInputKind::VariableStep => InputDefKind::VariableStep {
					max_value: self.max_value.unwrap() as usize,
					suggested_step: self.suggested_step.unwrap() as usize
				},
				RawInputKind::Action => InputDefKind::Action {
					argument: self.argument.clone().unwrap()
				}
			}
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RawOutputKind {
	String,
	Integer
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RawOutput {
	pub address: u16,
	pub description: String,
	#[serde(rename = "type")]
	pub kind: RawOutputKind,
	pub mask: Option<u16>,
	pub max_value: Option<u16>,
	pub shift_by: Option<u16>,
	pub max_length: Option<u16>,
	pub suffix: String
}

impl RawOutput {
	pub fn to_def(&self) -> OutputDef {
		OutputDef {
			description: self.description.clone(),
			kind: match self.kind {
				RawOutputKind::String => OutputDefKind::String {
					max_length: self.max_length.unwrap() as usize
				},
				RawOutputKind::Integer => OutputDefKind::Integer {
					max_value: self.max_value.unwrap() as usize
				}
			}
		}
	}

	pub fn read(&self, buffer: &[u8]) -> Option<Output> {
		let data = buffer.get((self.address as usize)..)?;
		match self.kind {
			RawOutputKind::String => {
				// how do we read a string
				let max_len = self.max_length.unwrap() as usize;
				let s = data.get(..max_len)?.split(|b| *b == 0).next()?;

				std::str::from_utf8(s).ok()
					.map(Into::into)
					.map(Output::String)
			},
			RawOutputKind::Integer => {
				let mut bytes = [0u8; 2];
				bytes.copy_from_slice(data.get(..2)?);
				let num = u16::from_le_bytes(bytes);
				let mask = self.mask.unwrap();
				let shift_by = self.shift_by.unwrap();

				let num = (num & mask) >> shift_by;

				Some(Output::Integer(num as i16))
			}
		}
	}
}