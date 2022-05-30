<script>
	import Mfcds from './mfcds.svelte';

	const pages = [
		{ name: 'MFCDS', comp: Mfcds }
	];

	let activePage = null;

	// debug
	activePage = pages[0];
</script>

{#if activePage}
	<svelte:component
		this={activePage.comp}
		on:close={() => activePage = null}
	/>
{:else}
	<h1>F-16C</h1>

	{#each pages as page}
		<a
			href="/page"
			on:click|preventDefault={() => activePage = page}
		>{page.name}</a>
	{/each}
{/if}