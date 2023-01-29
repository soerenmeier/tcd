<script>
	import { createEventDispatcher } from 'svelte';
	import Mfcd from './ui/mfcd.svelte';
	import BackBtn from './../../ui/back-btn.svelte';

	const dispatch = createEventDispatcher();

	let cont = null;

	const padding = 20;

	function maxSize() {
		const w = window.innerWidth / 2;
		const h = window.innerHeight;
		return Math.min(w, h) - padding;
	}
</script>

<div class="back-btn">
	<BackBtn on:click={() => dispatch('close')} />
</div>

<div id="mfcds" bind:this={cont}>
	<Mfcd size={maxSize} kind="LeftMfcd" name="MFD_L" />
	<div class="splitter"></div>
	<!-- <Mfcd size={maxSize} kind="RightMfcd" name="MFD_R" /> -->
</div>

<style>
	#mfcds {
		display: flex;
		height: 100%;
		padding: 10px;
		justify-content: space-between;
		align-items: center;
	}

	.splitter {
		height: 100%;
		width: 2px;
		background-color: var(--dark-gray);
	}

	.back-btn {
		position: absolute;
		top: 10px;
		left: 10px;
	}
</style>