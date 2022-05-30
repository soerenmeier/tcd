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
		display: block;
		width: 40px;
		height: 40px;
		background-color: var(--black);
		outline: none;
		border: none;
	}

	button.active {
		background-color: var(--white);
	}

	/*button:active {

	}*/
</style>