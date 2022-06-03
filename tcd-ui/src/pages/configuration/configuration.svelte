<script>
	import { createEventDispatcher } from 'svelte';
	import BackBtn from './../../ui/back-btn.svelte';

	const dispatch = createEventDispatcher();
	// we need to configure the displays
	// import Displays from './displays.svelte';
</script>

<div id="configuration">
	<div class="header">
		<BackBtn on:click={() => dispatch('close')} />
		<h1>Configuration</h1>
	</div>

	<!-- <Displays /> -->
</div>

<style>
	#configuration {
		padding: 20px;
	}

	.header {
		display: flex;
		gap: 10px;
		margin-bottom: 20px;
		align-items: center;
	}
</style>