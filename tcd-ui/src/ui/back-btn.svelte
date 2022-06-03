<script>
	import { createEventDispatcher } from 'svelte';

	const dispatch = createEventDispatcher();
</script>

<button
	class="no-styles"
	on:click={e => dispatch('click', e)}
></button>

<style>
	button {
		position: relative;
		width: 25px;
		height: 25px;
	}

	button::after {
		content: "";
		position: absolute;
		top: 6.5px;
		left: 9px;
		width: 10px;
		height: 10px;
		border-top: 2px solid var(--light-gray);
		border-left: 2px solid var(--light-gray);
		transform: rotate(-45deg);
	}
</style>