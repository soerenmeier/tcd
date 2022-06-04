<script>
	import { onDestroy } from 'svelte';
	import PushBtn from './../../../controls/push-btn.js';

	export let name;
	export let label = '';

	const btn = new PushBtn(name);

	onDestroy(() => {
		btn.destroy();
	});
</script>

<button
	on:click|preventDefault={() => btn.click()}
	class:active={$btn}
>
	{label}
</button>

<style>
	button {
		position: relative;
		display: block;
		width: 50px;
		height: 50px;
		background-color: var(--gray);
		outline: none;
		border: none;
		border-radius: 50%;
	}

	button.active {
		background-color: var(--dark-gray);
	}

</style>