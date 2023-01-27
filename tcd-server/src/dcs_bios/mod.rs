pub mod api;
mod stream;
use stream::Stream;
pub mod controls;
use controls::{ControlOutputs, Input};
pub mod control_definitions;
use control_definitions::ControlDefinitions;

use std::io;

use tokio::time::{self, Duration};
use tokio::task::JoinHandle;
use tokio::sync::{watch, mpsc};

#[derive(Debug, Clone)]
pub(crate) struct DcsBios {
	recv: watch::Receiver<ControlOutputs>,
	sender: mpsc::Sender<Input>
}

impl DcsBios {
	pub fn new(control_defs: ControlDefinitions) -> (Self, JoinHandle<()>) {
		let (tx, rx) = watch::channel(ControlOutputs::new());
		let (tx_2, rx_2) = mpsc::channel(20);

		let this = Self {
			recv: rx,
			sender: tx_2
		};

		let task = tokio::spawn(async move {
			let tx = tx;
			let mut rx = rx_2;

			loop {
				let r = stream_task(control_defs.clone(), &tx, &mut rx).await;
				match r {
					Ok(_) => break,
					Err(Error::Connecting(_)) => {
						time::sleep(Duration::from_secs(5)).await;
					},
					Err(e) => {
						eprintln!("dcs-bios error {:?}", e);
						time::sleep(Duration::from_secs(1)).await;
					}
				}
			}
		});

		(this, task)
	}

	pub async fn changed(&mut self) {
		self.recv.changed().await.expect("dcs-bios task failed");
	}

	pub fn borrow(&self) -> watch::Ref<'_, ControlOutputs> {
		self.recv.borrow()
	}

	pub async fn send(&self, input: Input) {
		self.sender.send(input).await.expect("dcs-bios task failed");
	}
}

#[derive(Debug)]
enum Error {
	Connecting(io::Error),
	Transmission(io::Error)
}

async fn stream_task(
	control_defs: ControlDefinitions,
	tx: &watch::Sender<ControlOutputs>,
	rx: &mut mpsc::Receiver<Input>
) -> Result<(), Error> {
	let mut stream = Stream::connect().await
		.map_err(Error::Connecting)?;

	let mut previous_loaded = String::new();

	eprintln!("connected to dcs bios");

	loop {
		// we need to use a state machine since we wan't to send
		// commands as fast as possible and not necessarilly wait until
		// the buffer is filled
		let maybe_buf = stream.read().await
			.map_err(Error::Transmission)?;
		if let Some(buf) = maybe_buf {

			let mut defs = control_defs.lock();

			let aicraft_outputs = defs.control_outputs(
				"_ACFT_NAME",
				buf
			);
			let aircraft = aicraft_outputs.into_string()
				.expect("missing aircraft");

			let loaded = defs.load_aircraft(&aircraft);
			if !loaded && !aircraft.is_empty() {
				eprintln!("could not load aircraft {}", aircraft);
				continue
			}

			if previous_loaded != aircraft {
				eprintln!("loaded aircraft {}", aircraft);
				previous_loaded = aircraft;
			}

			let outputs = defs.all_outputs(buf);

			drop(defs);

			tx.send_replace(outputs);
		}

		// check if we should send something
		if let Ok(input) = rx.try_recv() {
			let data = format!("{}\n", input);
			stream.write(data.as_bytes()).await
				.map_err(Error::Transmission)?;
		}
	}
}