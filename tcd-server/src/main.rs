
mod virtual_display;
use virtual_display::VirtualDisplay;
mod api_error;
mod mfds;
mod dcs_bios;
use dcs_bios::DcsBios;
use dcs_bios::control_definitions::ControlDefinitions;
mod displays;
use displays::{DisplaySetup, Displays};
#[cfg(feature = "self-host")]
mod web_api;

use fire::data_struct;

data_struct! {
	#[derive(Debug)]
	struct Data {
		virtual_display: VirtualDisplay,
		display_setup: DisplaySetup,
		control_defs: ControlDefinitions,
		dcs_bios: DcsBios
	}
}

#[tokio::main]
async fn main() {

	let display_setup = DisplaySetup::new();
	// for the moment let's just hard code these values
	display_setup.set(Some(Displays::default()));

	let (
		virtual_display,
		virtual_display_task
	) = VirtualDisplay::new(display_setup.clone());

	let control_defs = ControlDefinitions::new().await
		.expect("failed to open control definitions");

	let (dcs_bios, dcs_bios_task) = DcsBios::new(control_defs.clone());

	let data = Data {
		virtual_display,
		display_setup,
		control_defs,
		dcs_bios
	};

	let mut server = fire::build("0.0.0.0:3511", data).unwrap();

	mfds::handle(&mut server);
	dcs_bios::api::handle(&mut server);
	#[cfg(feature = "self-host")]
	web_api::handle(&mut server);

	eprintln!("!! Open http://127.0.0.1:3511 !!");

	tokio::try_join!(
		virtual_display_task,
		dcs_bios_task,
		tokio::spawn(async move {
			server.light().await
				.expect("server paniced");
		})
	).expect("one task failed");
}

