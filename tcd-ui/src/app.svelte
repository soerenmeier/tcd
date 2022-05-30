<script>
	import Errors from './ui/errors.svelte';
	import { newError } from './lib/errors.js';
	import Configuration from './pages/configuration/configuration.svelte';
	import F16C from './pages/f-16c/f-16c.svelte';

	const pages = [
		{ name: 'Configuration', comp: Configuration },
		{ name: 'F-16C', comp: F16C }
	];

	let activePage = pages[1];

	function onError(e) {
		// https://developer.mozilla.org/en-US/docs/Web/API/ErrorEvent
		newError('uncaught exception: ' + e.message);
	}

	function onUnhandleRejection(e) {
		// https://developer.mozilla.org/en-US/docs/Web/API/Window/unhandledrejection_event
		console.log(e);
		newError('unhadled rejection: ' + e.reason.message);
	}

</script>

<Errors />

<svelte:window
	on:resize={() => window.location.reload()}
	on:error={onError}
	on:unhandledrejection={onUnhandleRejection}
/>

<main class="full-size">
	<svelte:component this={activePage.comp} />
</main>

<style>
	main {
		overflow: hidden;
	}
</style>