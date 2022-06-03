<script>
	import { createEventDispatcher } from 'svelte';
	import Mfcd from './ui/mfcd.svelte';
	import BackBtn from './../../ui/back-btn.svelte';
	import Ded from './ui/ded.svelte';

	const dispatch = createEventDispatcher();

	let cont = null;

	const padding = 20;

</script>

<!-- <div class="back-btn"> -->
	<BackBtn on:click={() => dispatch('close')} />
<!-- </div> -->

<div id="icp">
	<h1>ICP</h1>

	<Ded />
</div>

<style>
	#icp {
		padding: 10px;
	}

	.back-btn {
		position: absolute;
		top: 10px;
		left: 10px;
	}
</style>