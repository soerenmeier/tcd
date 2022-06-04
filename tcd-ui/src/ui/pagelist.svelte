<script>
	import { createEventDispatcher } from 'svelte';
	import BackBtn from './back-btn.svelte';

	const dispatch = createEventDispatcher();
	// open, close

	// [{ name, comp }]
	export let pages;
	export let title;
	export let showBackBtn = false;
</script>

<div id="overview">
	<div class="header">
		{#if showBackBtn}
			<BackBtn on:click={() => dispatch('close')} />
		{/if}
		<h1>{title}</h1>
	</div>

	<div class="pages">
		{#each pages as page}
			<button
				class="page no-styles"
				on:click|preventDefault={() => dispatch('open', page)}
			>
				<span>{page.name}</span>
			</button>
		{/each}
	</div>
</div>

<style>
	#overview {
		padding: 20px;
	}

	.header {
		display: flex;
		gap: 10px;
		margin-bottom: 20px;
		align-items: center;
	}

	.pages {
		display: grid;
		grid-gap: 20px;
		grid-template-columns: repeat(6, 1fr);
	}

	.page {
		position: relative;
		display: block;
		width: 100%;
		padding-top: 100%;
		background-color: var(--gray);
		border-radius: 3px;
	}

	.page::after {
		content: '';
		position: absolute;
		top: 5%;
		left: 5%;
		width: 90%;
		height: 90%;
		box-sizing: border-box;
		border: 2px solid var(--light-gray);
		border-radius: 2px;
	}

	.page span {
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
	}

	@media (max-width: 800px) {
		.pages {
			grid-template-columns: repeat(3, 1fr);
		}
	}
</style>