import { subscribe, send, Input } from './../lib/controlsapi.js';
import { timeout } from 'fire/util.js';

export const DOWN = 0;
export const CENTER = 1;
export const UP = 2;

/// a button which has 3 states 0: Down, 1: Center, 2: Up
/// the state get's automatically reset to Center
export default class Push3WayBtn {
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
			if (this.state === val)
				return;
		}
	}

	async up() {
		send(Input.integer(this.name, 2));
		await this.onChange(2);
		send(Input.integer(this.name, 1));
	}

	async down() {
		send(Input.integer(this.name, 0));
		await this.onChange(0);
		send(Input.integer(this.name, 1));
	}

	destroy() {
		this._unsub();
	}
}