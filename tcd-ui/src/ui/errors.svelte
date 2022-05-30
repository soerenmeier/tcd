<script>
	import { errors } from './../lib/errors.js';

	let hasErrors = false;
	let errorList = [];

	function update(list) {
		hasErrors = list.length > 0;
		errorList = list;
	}
	$: update($errors);
</script>

{#if hasErrors}
	<div class="errors">
		{#each errorList as [id, error]}
			<div
				class="error"
				on:click={() => errors.remove(id)}
			>{error}</div>
		{/each}
	</div>
{/if}

<style>
	.errors {
		position: fixed;
		display: flex;
		bottom: 20px;
		right: 20px;
		width: 150px;
		flex-direction: column;
		gap: 5px;

		z-index: 999;
	}

	.error {
		background-color: var(--error-red);
		border-radius: 5px;
		cursor: pointer;
		padding: 5px 10px;
		color: var(--white);
	}
</style>