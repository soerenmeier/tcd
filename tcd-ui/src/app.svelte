<script>
	import Errors from './ui/errors.svelte';
	import Configuration from './pages/configuration/configuration.svelte';
	import F16C from './pages/f-16c/f-16c.svelte';

	const pages = [
		{ name: 'Configuration', comp: Configuration },
		{ name: 'F-16C', comp: F16C }
	];

	let activePage = pages[1];

</script>

<Errors />

<svelte:window on:resize={() => window.location.reload()} />

<main class="full-size">
	<svelte:component this={activePage.comp} />
</main>

<style>
	main {
		overflow: hidden;
	}
</style>