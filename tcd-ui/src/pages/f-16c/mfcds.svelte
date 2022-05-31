<script>
	import Mfcd from './ui/mfcd.svelte';

	let cont = null;

	const padding = 20;

	function maxSize() {
		const w = window.innerWidth / 2;
		const h = window.innerHeight;
		return Math.min(w, h) - padding;
	}
</script>

<div id="mfcds" bind:this={cont}>
	<Mfcd size={maxSize} kind="LeftMfcd" name="MFD_L" />
	<div class="splitter"></div>
	<Mfcd size={maxSize} kind="RightMfcd" name="MFD_R" />
</div>

<style>
	#mfcds {
		display: flex;
		height: 100%;
		padding: 10px;
		justify-content: space-between;
		align-items: center;
		background-color: var(--dark);
	}

	.splitter {
		height: 100%;
		width: 2px;
		background-color: var(--dark-gray);
	}
</style>