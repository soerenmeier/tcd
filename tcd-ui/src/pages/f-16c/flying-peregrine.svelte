<script>
	import { createEventDispatcher } from 'svelte';
	import BackBtn from './../../ui/back-btn.svelte';

	const dispatch = createEventDispatcher();
</script>

<div class="back-btn">
	<BackBtn on:click={() => dispatch('close')} />
</div>

<iframe src="https://flying-peregrine.com/f-16c"></iframe>

<style>
	iframe {
		width: 100%;
		height: 100%;
		outline: none;
		border: none;
	}

	.back-btn {
		position: absolute;
		top: 10px;
		left: 10px;
	}
</style>