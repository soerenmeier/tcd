// import { newError } from './errors.js';
import { getUrl } from './api.js';
import Api from 'fire/api/api.js';
import { timeout } from 'fire/util.js';

const api = new Api(getUrl('/').toString());

// once you don't the stream anymore call close
export async function newMfdWebrtc(kind, videoEl) {
	const pc = new RTCPeerConnection({
		iceServers: [{ urls: 'stun:stun.l.google.com:19302' }]
	});

	pc.addEventListener('track', e => {
		console.log('on track', e);

		videoEl.srcObject = e.streams[0];
		videoEl.autoplay = true;
	});

	pc.addEventListener('connectionstatechange', e => {
		console.log('connection state change', e);
	});

	pc.addEventListener('iceconnectionstatechange', e => {
		console.log('ice connection state change', e);
	});

	let resolveDescProm = () => {};
	let descProm = new Promise(res => resolveDescProm = res);

	pc.addEventListener('icecandidate', e => {
		console.log('icecandidate');
		if (e.candidate === null) {
			console.log('description', pc.localDescription);

			resolveDescProm(pc.localDescription);

			// console.log(sdp === pc.localDescription.sdp);
		}
	});

	pc.addTransceiver('video', { direction: 'recvonly' });

	const offer = await pc.createOffer({
		offerToReceiveVideo: 1
	});
	// console.log('got desc', desc);
	await pc.setLocalDescription(offer);
	// sdp = desc.sdp;

	const desc = await descProm;

	// await timeout(1000);

	// make the request and get the remote description
	const resp = await api.request('POST', 'mfd', { kind, desc });

	await pc.setRemoteDescription(new RTCSessionDescription(resp.desc));

	console.log('connection should be established');

	return pc;
}