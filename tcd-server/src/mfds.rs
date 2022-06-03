
use crate::Data;
use crate::api_error::Error;
use crate::displays::DisplayKind;

use std::collections::HashSet;

use serde::{Serialize, Deserialize};

use fire::{FireBuilder, ws_route};


#[derive(Debug, Clone, Serialize, Deserialize)]
enum Request {
	Subscribe(DisplayKind),
	Unsubscribe(DisplayKind),
	Aknowledge
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DisplayFramesAnnouncement {
	list: Vec<DisplayKind>
}

ws_route! {
	Mfds, "/api/mfds",
	|ws, virtual_display| -> Result<(), Error> {
		let mut subscribed = HashSet::new();
		let mut was_aknowledged = true;

		loop {
			tokio::select! {
				mut monitors = virtual_display.recv(&subscribed),
					if !subscribed.is_empty() && was_aknowledged
				=> {
					// send them
					let list: Vec<_> = monitors.keys().map(|k| *k).collect();
					if list.is_empty() {
						continue
					}

					let announcement = DisplayFramesAnnouncement {
						list: list.clone()
					};

					ws.serialize(&announcement).await
						.map_err(|e| Error::Internal(e.to_string()))?;

					for kind in list {
						let image = monitors.remove(&kind)
							// we always need to send data
							.unwrap_or_else(Vec::new);

						ws.send(image).await
							.map_err(|e| Error::Internal(e.to_string()))?;
					}

					was_aknowledged = false;
				},
				req = ws.deserialize() => {
					let maybe_req: Option<Request> = req
						.map_err(|e| Error::Internal(e.to_string()))?;
					let req = match maybe_req {
						Some(r) => r,
						// connection closed
						None => return Ok(())
					};

					match req {
						Request::Subscribe(kind) => {
							let _ = subscribed.insert(kind);
						},
						Request::Unsubscribe(kind) => {
							let _ = subscribed.remove(&kind);
						},
						Request::Aknowledge => {
							was_aknowledged = true;
						}
					}
				}
			}
		}
	},
	|ret| {
		if let Err(e) = ret {
			eprintln!("ws mfds error {}", e);
		}
	}
}

pub(crate) fn handle(fire: &mut FireBuilder<Data>) {
	fire.add_raw_route(Mfds);
}