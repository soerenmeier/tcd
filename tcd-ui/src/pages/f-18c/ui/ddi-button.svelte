<script>
	import { onDestroy } from 'svelte';
	import PushBtn from './../../../controls/push-btn.js';

	export let name;

	const btn = new PushBtn(name);

	onDestroy(() => {
		btn.destroy();
	});
</script>

<button
	on:click|preventDefault={() => btn.click()}
	class:active={$btn}
></button>

<style>
	button {
		position: relative;
		display: block;
		width: 40px;
		height: 40px;
		background-color: var(--light-gray);
		outline: none;
		border: none;
		border-radius: 3px;

		/* vars for after  */
		--space: 8px;
		--width: calc(40px - 16px);
	}

	button.active {
		background-color: var(--dark-gray);
	}

	button::after {
		content: "";
		position: absolute;
		background-color: var(--white);
	}

	:global(.top) > button::after,
	:global(.bottom) > button::after {
		top: var(--space);
		left: calc(50% - 1.5px);
		width: 3px;
		height: var(--width);
	}

	:global(.left) > button::after,
	:global(.right) > button::after {
		top: calc(50% - 1.5px);
		left: var(--space);
		width: var(--width);
		height: 3px;
	}
</style>