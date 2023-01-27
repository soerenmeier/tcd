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

	let mut server = fire::build("0.0.0.0:3511").await.unwrap();

	server.add_data(virtual_display);
	server.add_data(display_setup);
	server.add_data(control_defs);
	server.add_data(dcs_bios);

	mfds::handle(&mut server);
	dcs_bios::api::handle(&mut server);
	#[cfg(feature = "self-host")]
	web_api::handle(&mut server);

	eprintln!("!! Open http://127.0.0.1:3511 !!");

	tokio::try_join!(
		virtual_display_task,
		dcs_bios_task,
		tokio::spawn(async move {
			server.ignite().await
				.expect("server paniced");
		})
	).expect("one task failed");
}