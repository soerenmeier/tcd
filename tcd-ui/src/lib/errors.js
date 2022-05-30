
import SlotArray from 'fire/util/slotarray.js';
import ApiError from 'fire/api/error.js';

class Errors {
	constructor() {
		this.actives = new SlotArray;
		this.listeners = new Set;
	}

	get() {
		return this.actives.entries();
	}

	/// fn([[id, str]])
	subscribe(fn) {
		this.listeners.add(fn);
		fn(this.actives.entries());

		return () => {
			this.listeners.delete(fn);
		};
	}

	push(val) {
		this.actives.push(val);
		this.notify();
	}

	/// after calling remove once your not allowed to call it again
	/// because the id might be given to some other error
	remove(id) {
		this.actives.remove(id);
		this.notify();
	}

	notify() {
		const entries = this.actives.entries();
		this.listeners.forEach(fn => fn(entries));
	}
}

export const errors = new Errors;

export function newError(value) {
	errors.push(value);
}

export async function fromException(fn) {
	try {
		return await fn();
	} catch (e) {
		if (e instanceof ApiError)
			newError(e.kind + ': ' + e.msg);
		else
			newError('Unknown error occured');

		throw e;
	}
}