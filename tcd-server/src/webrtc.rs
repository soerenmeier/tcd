// mod twcc_interceptor;

// use twcc_interceptor::TwccInterceptor;

use crate::displays::FrameReceiver;
use crate::buffers::Buffer;

use std::mem;
use std::time::{Duration, Instant};
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::task::spawn_blocking;

use simple_bytes::{BytesWrite, BytesRead};

use webrtc::api::interceptor_registry::{
	configure_nack, configure_rtcp_reports, configure_twcc
};
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_H264};
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_connection_state::RTCIceConnectionState;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::ice_transport::ice_candidate::RTCIceCandidate;
use webrtc::interceptor::registry::Registry;
use webrtc::media::Sample;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
pub use webrtc::peer_connection::sdp::session_description::{
	RTCSessionDescription as Description
};
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::{
	TrackLocalStaticSample
};
use webrtc::track::track_local::TrackLocal;
use webrtc::stats::StatsReportType;

use openh264::encoder::{EncoderConfig, Encoder, RateControlMode};
use openh264::formats::YUVSource;


#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("webrtc error")]
	WebrtcError(#[from] webrtc::Error),
	#[error("open h264 error")]
	Encoder(#[from] openh264::Error),
	#[error("blocking task failed")]
	JoinError(#[from] tokio::task::JoinError)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
	Connected,
	Disconnected
}

pub struct Webrtc {}

impl Webrtc {
	pub fn new() -> Self {
		Self {}
	}

	pub async fn create_connection(
		&self,
		desc: Description,
		frames: FrameReceiver
	) -> Result<Connection, Error> {
		let mut m = MediaEngine::default();

		// cannot fail, see webrtc impl
		m.register_default_codecs().unwrap();

		let mut registry = Registry::new();

		registry = configure_nack(registry, &mut m);
		registry = configure_rtcp_reports(registry);
		registry = configure_twcc(registry, &mut m)?;
		// registry.add(Box::new(TwccInterceptor::builder()));

		let api = APIBuilder::new()
			.with_media_engine(m)
			.with_interceptor_registry(registry)
			.build();

		let config = RTCConfiguration {
			ice_servers: vec![RTCIceServer {
				urls: vec!["stun:stun.l.google.com:19302".to_owned()],
				..Default::default()
			}],
			..Default::default()
		};

		let peer_connection = Arc::new(api.new_peer_connection(config).await?);

		let video_track = Arc::new(TrackLocalStaticSample::new(
			RTCRtpCodecCapability {
				mime_type: MIME_TYPE_H264.to_owned(),
				..Default::default()
			},
			"video".to_owned(),
			"webrtc-rs".to_owned()
		));

		let rtp_sender = peer_connection.add_track(
			Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>
		).await?;

		let (state_tx, state_rx) = mpsc::channel(5);

		let n_peer_connection = peer_connection.clone();
		tokio::spawn(async move {
			eprintln!("start frame task");

			let r = frame_task(
				frames,
				video_track,
				n_peer_connection,
				state_rx
			).await;

			if let Err(e) = r {
				tracing::error!("frame task error {e:?}");
			}

			eprintln!("frame task closed");
		});

		// keep the rtp stream going
		tokio::spawn(async move {
			let mut rtcp_buf = vec![0u8; 3000];
			while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
		});

		// set the handler for ICE connection
		peer_connection.on_ice_connection_state_change(Box::new(
			move |connection_state: RTCIceConnectionState| {
				eprintln!("ice connection state changed {:?}", connection_state);
				// if connection_state == RTCIceConnectionState::Connected {
				// 	notify_tx.notify_waiters();
				// }

				Box::pin(async {})
			}
		));

		peer_connection.on_ice_candidate(Box::new(
			move |c: Option<RTCIceCandidate>| {
				eprintln!("ice candidate update {:?}", c);

				Box::pin(async {})
			}
		));

		peer_connection.on_peer_connection_state_change(
			Box::new(move |s: RTCPeerConnectionState| {
				eprintln!("peer connection state change {:?}", s);

				match s {
					RTCPeerConnectionState::Connected => {
						let _ = state_tx.try_send(State::Connected);
					},
					RTCPeerConnectionState::Disconnected |
					RTCPeerConnectionState::Failed |
					RTCPeerConnectionState::Closed => {
						let _ = state_tx.try_send(State::Disconnected);
					},
					_ => {}
				}

				Box::pin(async {})
			})
		);

		peer_connection.set_remote_description(desc).await?;

		let answer = peer_connection.create_answer(None).await?;

		peer_connection.set_local_description(answer).await?;

		Ok(Connection { peer_connection })
	}
}

pub struct Connection {
	peer_connection: Arc<RTCPeerConnection>
}

impl Connection {
	pub async fn description(&self) -> Description {
		self.peer_connection.local_description().await.unwrap()
	}

	#[allow(dead_code)]
	pub async fn close(&self) {
		self.peer_connection.close().await.unwrap();
	}
}

// every x frames
const GATHER_STATS_EVERY: usize = 15;

const FRAME_RATE: Duration = Duration::from_millis(1000 / 30);

async fn frame_task(
	mut frames: FrameReceiver,
	track: Arc<TrackLocalStaticSample>,
	peer_connection: Arc<RTCPeerConnection>,
	mut state_rx: mpsc::Receiver<State>
) -> Result<(), Error> {
	// let's wait until the connection is established
	match state_rx.recv().await {
		Some(State::Connected) => {},
		Some(State::Disconnected) |
		None => return Ok(())
	};

	eprintln!("start frame handling");


	let mut gather_stats_in = 0;
	let mut stats;

	let display = frames.display();
	let config = EncoderConfig::new(display.width, display.height)
		.set_bitrate_bps(60_000);

	let mut encoder = Encoder::with_config(config)
		.map_err(Error::Encoder)?;



	// let mut last_yuv = None;
	let mut last_sample_sent = Instant::now();
	let mut nals = vec![];
	// let mut bytes = BytesMut::new();
	// let mut interval = interval(FRAME_RATE).await;
	// interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

	// let's try to get some information
	loop {
		// interval.tick().await;

		if gather_stats_in == 0 {
			stats = peer_connection.get_stats().await;
			// for (key, stat) in stats.reports {
			// 	// let stat = match stat {
			// 	// 	StatsReportType::CandidatePair(stat) |
			// 	// 	StatsReportType::LocalCandidate(stat) |
			// 	// 	StatsReportType::RemoteCandidate(stat)
			// 	// }
			// 	if let StatsReportType::CandidatePair(stat) = stat {
			// 		eprintln!("{:?} {:?} {:?}", key, stat.available_incoming_bitrate, stat.available_outgoing_bitrate);
			// 	}
			// }
			gather_stats_in = GATHER_STATS_EVERY;
		} else {
			gather_stats_in -= 1;
		}

		// check if the connection already closed
		match state_rx.try_recv() {
			Ok(State::Connected) => unreachable!(),
			Ok(State::Disconnected) |
			Err(mpsc::error::TryRecvError::Disconnected) => return Ok(()),
			Err(mpsc::error::TryRecvError::Empty) => {}
		}

		// let bytes = match frames.try_recv() {

		// }

		let Some((buffer, missed_nals)) = frames.recv().await else {
			eprintln!("display closed no more nals");
			// we will not receive any frames
			return Ok(())
		};

		tracing::info!("sent buffer");

		if missed_nals > 0 {
			eprintln!("received nals to late, missed {missed_nals:?}");
		}

		let display = frames.display().clone();
		

		let mut n_nals = mem::take(&mut nals);
		(encoder, nals) = spawn_blocking(move || {
			let yuv = YuvData::new(
				buffer.as_slice(),
				display.width,
				display.height
			);

			let encoded = encoder.encode(&yuv)?;

			// write all nals
			for layer in 0..encoded.num_layers() {
				let layer = encoded.layer(layer).unwrap();
				for nal in 0..layer.nal_count() {
					let nal = layer.nal_unit(nal).unwrap();

					// write the nal
					let mut buffer = Buffer::new();
					buffer.write(nal);

					n_nals.push(buffer.into_shared().into_bytes());
				}
			}

			Ok::<_, Error>((encoder, n_nals))
		}).await??;

		// write all nals
		for nal in nals.drain(..) {
			let sample = Sample {
				data: nal,
				duration: last_sample_sent.elapsed(),
				..Default::default()
			};

			track.write_sample(&sample).await?;
		}

		last_sample_sent = Instant::now();
	}
}

struct YuvData<'a> {
	inner: &'a [u8],
	width: u32,
	height: u32
}

impl<'a> YuvData<'a> {
	pub fn new(inner: &'a [u8], width: u32, height: u32) -> Self {
		assert_eq!(inner.len() as u32, (width * height * 3) / 2);

		Self { inner, width, height }
	}

	fn frame_size(&self) -> usize {
		self.width as usize * self.height as usize
	}
}

impl YUVSource for YuvData<'_> {
	fn width(&self) -> i32 {
		self.width as i32
	}

	fn height(&self) -> i32 {
		self.height as i32
	}

	fn y(&self) -> &[u8] {
		&self.inner[..self.frame_size()]
	}

	fn u(&self) -> &[u8] {
		let fs = self.frame_size();
		&self.inner[fs..][..fs / 4]
	}

	fn v(&self) -> &[u8] {
		let fs = self.frame_size();
		&self.inner[(fs + fs / 4)..]
	}

	fn y_stride(&self) -> i32 {
		self.width as i32
	}

	fn u_stride(&self) -> i32 {
		(self.width / 2) as i32
	}

	fn v_stride(&self) -> i32 {
		(self.width / 2) as i32
	}
}