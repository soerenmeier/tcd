<script>
	import { createEventDispatcher } from 'svelte';
	import Ddis from './ddis.svelte';
	import PageList from './../../ui/pagelist.svelte';

	const dispatch = createEventDispatcher();

	const pages = [
		{ name: 'DDIS', comp: Ddis }
	];

	let activePage = null;

	// debug
	// activePage = pages[0];
</script>

{#if activePage}
	<svelte:component
		this={activePage.comp}
		on:close={() => activePage = null}
	/>
{:else}
	<PageList
		{pages}
		title="F-18C"
		showBackBtn={true}
		on:close={() => dispatch('close')}
		on:open={e => activePage = e.detail}
	/>
{/if}