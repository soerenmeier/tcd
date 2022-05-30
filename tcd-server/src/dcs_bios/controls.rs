
use std::fmt;
use std::collections::HashMap;

use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "camelCase")]
pub struct ControlDefs {
	controls: HashMap<String, ControlDef>
}

impl ControlDefs {
	pub fn new() -> Self {
		Self {
			controls: HashMap::new()
		}
	}

	pub fn insert(&mut self, name: String, def: ControlDef) {
		self.controls.insert(name, def);
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "camelCase")]
pub struct ControlDef {
	pub category: String,
	pub kind: String,
	pub description: String,
	pub inputs: Vec<InputDef>,
	pub outputs: Vec<OutputDef>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "camelCase")]
pub struct InputDef {
	pub description: String,
	pub kind: InputDefKind
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputDefKind {
	FixedStep,
	SetState {
		max_value: usize
	},
	VariableStep {
		max_value: usize,
		suggested_step: usize
	},
	Action {
		argument: String
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "camelCase")]
pub struct OutputDef {
	pub description: String,
	pub kind: OutputDefKind
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputDefKind {
	String {
		max_length: usize
	},
	Integer {
		max_value: usize
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ControlOutputs {
	inner: HashMap<String, Outputs>
}

impl ControlOutputs {
	pub fn new() -> Self {
		Self {
			inner: HashMap::new()
		}
	}

	pub fn get(&self, name: &str) -> Option<&Outputs> {
		self.inner.get(name)
	}

	pub fn insert(&mut self, name: String, outputs: Outputs) {
		self.inner.insert(name, outputs);
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Outputs {
	inner: Vec<Output>
}

impl Outputs {
	pub fn new() -> Self {
		Self {
			inner: vec![]
		}
	}

	pub fn into_string(self) -> Option<String> {
		self.inner.into_iter().find_map(|o| match o {
			Output::String(s) => Some(s),
			_ => None
		})
	}
}

impl From<Vec<Output>> for Outputs {
	fn from(inner: Vec<Output>) -> Self {
		Self { inner }
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Output {
	String(String),
	Integer(i16)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
	name: String,
	value: InputValue
}

impl fmt::Display for Input {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{} {}", self.name, self.value)
	}
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InputValue {
	Increase,
	Decrease,
	Toggle,
	Integer(i16)
}

impl fmt::Display for InputValue {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Increase => write!(f, "INC"),
			Self::Decrease => write!(f, "DEC"),
			Self::Toggle => write!(f, "TOGGLE"),
			Self::Integer(i) => write!(f, "{}", i)
		}
	}
}

// enum ControlKind {
// 	/// if set_state:max_value = 3 (is a 3 way switch)
// 	/// if set_state:max_value > 4 (rotary dial)
// 	/// if set_state:max_value == 1 (has probably toggle)
// 	Selector,
// 	/// mostly output (integer)
// 	AnalogGauge,
// 	/// mostly input (variable_step) output (integer)
// 	AnalogDial,
// 	/// mostly output (integer/string)
// 	Display,
// 	EmergencyParkingBrake,
// 	Led,
// 	MissionComputerSwitch,
// 	LimitedDial,
// 	DiscreteDial,
// 	FixedStepDial,
// 	ToggleSwitch,
// 	Action,
// 	Metadata,
// 	Frequency,
// 	// 3Pos2CommandSwitch,
// 	// 3Pos2CommandSwitchOpenClose,
// 	VariableStepDial
// }