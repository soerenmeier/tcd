<script>
	import { createEventDispatcher } from 'svelte';
	import PageList from './../../ui/pagelist.svelte';

	const dispatch = createEventDispatcher();

	const pages = [];

	let activePage = null;
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