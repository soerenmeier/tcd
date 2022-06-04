<script>
	import { onDestroy } from 'svelte';
	import PushBtn from './../../../controls/push-btn.js';

	export let name;
	export let label = '';
	export let number = null;
	export let small = null;

	const btn = new PushBtn(name);

	onDestroy(() => {
		btn.destroy();
	});
</script>

<button
	on:click|preventDefault={() => btn.click()}
	class:active={$btn}
	class:has-number={number !== null}
>
	<span class="label">{label}</span>
	{#if number !== null}
		<span class="number">{number}</span>
	{/if}
	{#if small !== null}
		<span class="small">{small}</span>
	{/if}
</button>

<style>
	button {
		position: relative;
		display: block;
		width: 55px;
		height: 55px;
		background-color: var(--gray);
		outline: none;
		border: none;
		border-radius: 3px;
	}

	button.active {
		background-color: var(--dark-gray);
	}

	.label {
		display: block;
		font-size: 14px;
	}

	.number {
		display: block;
		font-weight: 800;
	}

	.small {
		position: absolute;
		font-size: 10px;
		bottom: 4px;
		right: 4px;
	}

</style>