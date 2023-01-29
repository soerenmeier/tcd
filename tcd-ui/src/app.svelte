<script>
	import Errors from './ui/errors.svelte';
	import { newError } from './lib/errors.js';
	import { subscribe } from './lib/controlsapi.js';
	import PageList from './ui/pagelist.svelte';

	import Configuration from './pages/configuration/configuration.svelte';
	import F16C from './pages/f-16c/f-16c.svelte';
	// import F18C from './pages/f-18c/f-18c.svelte';

	const pages = [
		{ id: 'config', name: 'Configuration', comp: Configuration },
		{ id: 'F-16C_50', name: 'F-16C', comp: F16C },
		// { id: 'FA-18C_hornet', name: 'F-18C', comp: F18C }
	];

	let activePage = null;

	function onError(e) {
		// https://developer.mozilla.org/en-US/docs/Web/API/ErrorEvent
		newError('uncaught exception: ' + e.message);
	}

	function onUnhandleRejection(e) {
		// https://developer.mozilla.org/en-US/docs/Web/API/Window/unhandledrejection_event
		console.log(e);
		newError('unhadled rejection: ' + e.reason.message);
	}

	let currentAircraft = null;
	subscribe('_ACFT_NAME', outs => {
		if (!outs)
			return;

		const s = outs.string();
		if (currentAircraft == s)
			return;

		currentAircraft = s;

		// check if we have this page
		const p = pages.find(p => p.id === currentAircraft);
		if (p)
			activePage = p;
		else
			console.log('aircraft not found', currentAircraft);
	});

	// debug
	// activePage = pages[2];
</script>

<Errors />

<svelte:window
	on:resize={() => window.location.reload()}
	on:error={onError}
	on:unhandledrejection={onUnhandleRejection}
/>

<main class="full-size">
	{#if activePage}
		<svelte:component
			this={activePage.comp}
			on:close={() => activePage = null}
		/>
	{:else}
		<PageList
			{pages}
			title="Pages"
			on:open={e => activePage = e.detail}
		/>
	{/if}
</main>

<style>
	main {
		overflow: hidden;
		background-color: var(--dark);
		color: var(--white);
	}
</style>