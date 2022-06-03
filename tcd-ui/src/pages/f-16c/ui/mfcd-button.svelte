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
>
	<div class="stroke"></div>
</button>

<style>
	button {
		position: relative;
		display: block;
		width: 40px;
		height: 40px;
		background-color: var(--gray);
		outline: none;
		border: none;
		border-radius: 3px;
	}

	button.active {
		background-color: var(--dark-gray);
	}

	.stroke {
		position: absolute;
		top: 10%;
		left: 10%;
		width: 80%;
		height: 80%;
		border: 2px solid var(--light-gray);
		border-radius: 2px;
	}

	.active .stroke {
		border: 2px solid var(--gray);
	}

	/*button:active {

	}*/
</style>