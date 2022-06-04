import { subscribe, send, Input } from './../../../lib/controlsapi.js';
import { timeout } from 'fire/util.js';

export const CENTER = 0;
export const UP = 1;
export const RIGHT = 2;
export const DOWN = 3;
export const LEFT = 4;

export default class IcpDataControlSwitch {
	constructor() {
		// 0=center, 1=top, 2=right, 3=bottom, 4=left
		this.state = null;

		this._listeners = new Set;

		let state1 = 1;
		let state2 = 1;
		this._unsub1 = subscribe('ICP_DATA_RTN_SEQ_SW', resp => {
			if (!resp)
				return;

			const s = resp.integer();
			state1 = s;

			if (s == 0)// down
				this.state = 4;
			else if (s == 2)// up
				this.state = 2;
			else if (s == 1) { // center
				// since we have two buttons which 'fight' for who has to say
				// what happens
				// they need to agree if the switch is at the center
				if (state2 == 1)
					this.state = 0;
			}
			this._listeners.forEach(fn => fn(this.state));
		});
		this._unsub2 = subscribe('ICP_DATA_UP_DN_SW', resp => {
			if (!resp)
				return;

			const s = resp.integer();
			state2 = s;

			if (s == 0)// left
				this.state = 3;
			else if (s == 2)// right
				this.state = 1;
			else if (s == 1 && state1 == 1)// center
				this.state = 0;
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
		return await this.setState(1);
	}

	async right() {
		return await this.setState(2);
	}

	async down() {
		return await this.setState(3);
	}

	async left() {
		return await this.setState(4);
	}

	// priv
	async setState(num) {
		let name, state;
		if (num == 2 || num == 4) {
			name = 'ICP_DATA_RTN_SEQ_SW';
			state = num == 2 ? 2 : 0;
		} else if (num == 1 || num == 3) {
			name = 'ICP_DATA_UP_DN_SW';
			state = num == 1 ? 2 : 0;
		}

		send(Input.integer(name, state));
		// wait until the right state is set
		await this.onChange(num);
		// then but the switch to the center again
		send(Input.integer(name, 1));
	}

	destroy() {
		this._unsub1();
		this._unsub2();
	}
}