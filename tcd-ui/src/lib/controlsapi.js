
import { newError } from './errors.js';
import { getUrl } from './api.js';
import Data from 'fire/data/data.js';

let failed = false;
let ws = null;
let listeners = new Map;// Map<Kind, Set>

export class Response extends Data {
	constructor(d) {
		super({
			name: 'str',
			outputs: [Output]
		}, d);
	}

	integer() {
		return this.outputs.find(o => o.kind === 'Integer')?.value ?? null;
	}

	string() {
		return this.outputs.find(o => o.kind === 'String')?.value ?? null;
	}
}

export class Output extends Data {
	constructor(d) {
		super();
		if ('String' in d) {
			this.kind = 'String';
			this.value = d.String;
		} else if ('Integer' in d) {
			this.kind = 'Integer';
			this.value = d.Integer;
		} else {
			throw new Error('unknown Kind');
		}
	}
}

export class Input extends Data {
	constructor(d) {
		super({
			name: 'str',
			value: InputValue
		}, d);
	}

	static new(name, value) {
		const self = Object.create(this.prototype);
		self.name = name;
		self.value = value;

		return self;
	}

	static increase(name) {
		return Input.new(name, InputValue.new('Increase'));
	}

	static decrease(name) {
		return Input.new(name, InputValue.new('Decrease'));
	}

	static toggle(name) {
		return Input.new(name, InputValue.new('Toggle'));
	}

	static integer(name, num) {
		return Input.new(name, InputValue.new('Integer', num));
	}
}

export class InputValue extends Data {
	constructor(d) {
		if (typeof d === 'string') {
			this.kind = d;
			this.value = null;
		} else if ('Integer' in d) {
			this.kind = 'Integer';
			this.value = d.Integer;
		} else {
			throw new Error('unknown Kind');
		}
	}

	static new(kind, value = null) {
		const self = Object.create(this.prototype);
		self.kind = kind;
		self.value = value;

		return self;
	}

	toJSON() {
		if (this.value !== null) {
			const o = {};
			o[this.kind] = this.value;
			return o;
		}

		return this.kind;
	}
}


export function subscribe(name, fn) {
	if (failed)
		throw new Error('cannot subscribe websocket connection failed');

	if (listeners.has(name)) {
		listeners.get(name).add(fn);
	} else {
		const set = new Set;
		set.add(fn);
		listeners.set(name, set);

		// need to send subscribe
		if (ws && ws.readyState === 1)
			sendSubscribe(name);
		// if the connection is not ready
		// initWs will automatically subscribe to all kinds that are in
		// listeners
	}

	fn(null);

	if (!ws)
		initWs();

	return () => {
		let set = listeners.get(name);
		set.delete(fn);

		if (set.size === 0) {
			listener.delete(name);
			ws.send(JSON.stringify({ 'Unsubscribe': name }));
		}

		// if (listeners.size === 0)
		// 	closeWs();
	};
}

// send needs to be of Input type
export function send(input) {
	if (failed)
		throw new Error('websocket connection failed');

	ws.send(JSON.stringify({
		Input: input
	}));
}

// you need to make sure that connection is active
function sendSubscribe(kind) {
	ws.send(JSON.stringify({ 'Subscribe': kind }));
}

function notify(kind, value) {
	const set = listeners.get(kind);
	if (!set)
		return;
	set.forEach(fn => fn(value));
}

function initWs() {
	let url = getUrl('/controls/stream');
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

		newError('Controls stream closed');
	});

	ws.addEventListener('message', wsMsg => {
		const d = JSON.parse(wsMsg.data);

		const resp = new Response(d);

		const set = listeners.get(resp.name);
		if (set)
			set.forEach(fn => fn(resp));
	});
}

function closeWs() {
	if (!ws)
		return;

	ws.close();
	ws = null;
}