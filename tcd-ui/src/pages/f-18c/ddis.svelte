<script>
	import { createEventDispatcher } from 'svelte';
	import Ddi from './ui/ddi.svelte';
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

<div id="ddis" bind:this={cont}>
	<Ddi size={maxSize} kind="LeftMfcd" name="LEFT_DDI_PB" />
	<div class="splitter"></div>
	<Ddi size={maxSize} kind="RightMfcd" name="RIGHT_DDI_PB" />
</div>

<style>
	#ddis {
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