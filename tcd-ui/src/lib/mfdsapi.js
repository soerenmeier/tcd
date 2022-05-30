
import { newError } from './errors.js';
import { getUrl } from './api.js';

let failed = false;
let ws = null;
let listeners = new Map;// Map<Kind, Set>
let currentFrames = new Map;

export function subscribe(kind, fn) {
	if (failed)
		throw new Error('cannot subscribe websocket connection failed');

	if (listeners.has(kind)) {
		listeners.get(kind).add(fn);
	} else {
		const set = new Set;
		set.add(fn);
		listeners.set(kind, set);

		// need to send subscribe
		if (ws && ws.readyState === 1)
			sendSubscribe(kind);
		// if the connection is not ready
		// initWs will automatically subscribe to all kinds that are in
		// listeners
	}

	fn(currentFrames.get(kind) ?? null);

	if (!ws)
		initWs();

	return () => {
		let set = listeners.get(kind);
		set.delete(fn);

		if (set.size === 0) {
			listener.delete(kind);
			sendSubscribe(kind);
		}

		// if (listeners.size === 0)
		// 	closeWs();
	};
}

// you need to make sure that connection is active
function sendSubscribe(kind) {
	ws.send(JSON.stringify({ 'Subscribe': kind }));
}

function notify(kind) {
	const set = listeners.get(kind);
	if (!set)
		return;
	set.forEach(fn => fn(currentFrames.get(kind) ?? null));
}

function initWs() {
	let url = getUrl('/mfds');
	url.protocol = 'ws:';
	ws = new WebSocket(url);

	ws.addEventListener('open', e => {
		// the connection was opened
		// we now may need to call subscribe
		for (const kind of listeners.keys()) {
			sendSubscribe(kind);
		}
	});

	ws.addEventListener('close', e => {
		ws = null;
		failed = true;

		newError('Mfds stream closed');
	});

	let missingKinds = [];
	ws.addEventListener('message', wsMsg => {
		// we expect a frames announcement
		if (missingKinds.length === 0) {
			const d = JSON.parse(wsMsg.data);
			if (typeof d !== 'object' || !('list' in d)) {
				console.log('received unexpected message', d);
				throw new Error('invalid message');
			}

			// reverse it so we can just call pop
			missingKinds = d.list.reverse();
			return;
		}

		const kind = missingKinds.pop();

		// send aknowledge if we received all kinds
		// this tells the server it can send another frame
		if (missingKinds.length === 0)
			ws.send(JSON.stringify('Aknowledge'));

		// we expect it to be a jpeg blob
		let blob = wsMsg.data;
		blob = blob.slice(0, blob.size, 'image/jpeg');
		const reader = new FileReader;
		reader.addEventListener('load', () => {
			const img = new Image;
			img.src = reader.result;
			img.addEventListener('load', () => {
				currentFrames.set(kind, img);

				notify(kind);
			});
			
		});
		reader.readAsDataURL(blob);
	});
}

function closeWs() {
	if (!ws)
		return;

	ws.close();
	ws = null;
}