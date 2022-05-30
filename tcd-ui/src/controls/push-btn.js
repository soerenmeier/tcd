import { subscribe, send, Input } from './../lib/controlsapi.js';
import { timeout } from 'fire/util.js';

export default class PushBtn {
	constructor(name) {
		this.name = name;
		this.state = null;

		this._listeners = new Set;

		this._unsub = subscribe(name, resp => {
			if (!resp)
				return;

			this.state = resp.integer();
			this._listeners.forEach(fn => fn(this.state));
		});
	}

	subscribe(fn) {
		this._listeners.add(fn);
		fn(this.state);

		return () => {
			this._listeners.delete(fn);
		};
	}

	async onChange(val) {
		while (true) {
			const prom = new Promise(resolve => {
				const fn = d => {
					this._listeners.delete(fn);
					resolve(d);
				}
				this._listeners.add(fn);
			});

			await prom;
			if (this.state !== val)
				continue;
			return;
		}
	}

	async click() {
		send(Input.integer(this.name, 1));
		await this.onChange(1);
		send(Input.integer(this.name, 0));
	}

	destroy() {
		this._unsub();
	}
}