
use super::controls::{Input, Outputs};
use crate::Data;
use crate::api_error::Error;

use std::collections::HashSet;

use serde::{Serialize, Deserialize};

use fire::{FireBuilder, ws_route};


#[derive(Debug, Clone, Serialize, Deserialize)]
enum Request {
	Subscribe(String),
	Unsubscribe(String),
	Input(Input),
	Aknowledge
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Announce {
	pub len: u32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Response {
	pub name: String,
	pub outputs: Outputs
}

ws_route! {
	WsApi, "/api/controls/stream",
	|ws, dcs_bios| -> Result<(), Error> {
		let mut subscribed: HashSet<String> = HashSet::new();
		let mut was_aknowledged = true;

		loop {
			tokio::select! {
				_ = dcs_bios.changed(),
					if !subscribed.is_empty() && was_aknowledged
				=> {
					// we need to store the responses before sending
					// to hold the watch Lock as short as possible
					let mut responses = Vec::with_capacity(subscribed.len());
					{
						let outputs = dcs_bios.borrow();
						for name in &subscribed {
							if let Some(outputs) = outputs.get(name) {
								responses.push(Response {
									name: name.clone(),
									outputs: outputs.clone()
								});
							}
						}
					}

					ws.serialize(&Announce {
						len: responses.len() as u32
					}).await
						.map_err(|e| Error::Internal(e.to_string()))?;

					for response in responses {
						ws.serialize(&response).await
							.map_err(|e| Error::Internal(e.to_string()))?;
					}

					was_aknowledged = false;
				},
				req = ws.deserialize() => {
					let req = req.map_err(|e| Error::Internal(e.to_string()))?;
					let req = match req {
						Some(r) => r,
						None => return Ok(())
					};

					match req {
						Request::Subscribe(name) => {
							subscribed.insert(name);
						},
						Request::Unsubscribe(name) => {
							subscribed.remove(&name);
						},
						Request::Input(inp) => {
							dcs_bios.send(inp).await;
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
			eprintln!("ws controls error {}", e);
		}
	}
}

pub(crate) fn handle(fire: &mut FireBuilder<Data>) {
	fire.add_raw_route(WsApi);
}