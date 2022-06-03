<script>
	import { createEventDispatcher } from 'svelte';
	import Mfcds from './mfcds.svelte';
	import FlyingPeregrine from './flying-peregrine.svelte';
	import Icp from './icp.svelte';
	import PageList from './../../ui/pagelist.svelte';

	const dispatch = createEventDispatcher();

	const pages = [
		{ name: 'MFCDS', comp: Mfcds },
		{ name: 'ICP', comp: Icp },
		{ name: 'Flying Peregrine', comp: FlyingPeregrine }
	];

	let activePage = null;

	// debug
	activePage = pages[1];
</script>

{#if activePage}
	<svelte:component
		this={activePage.comp}
		on:close={() => activePage = null}
	/>
{:else}
	<PageList
		{pages}
		title="F-16C"
		showBackBtn={true}
		on:close={() => dispatch('close')}
		on:open={e => activePage = e.detail}
	/>
{/if}