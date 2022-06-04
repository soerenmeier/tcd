<script>
	import { onDestroy } from 'svelte';
	import Push3WayBtn, {
		DOWN, CENTER, UP
	} from './../../../controls/push-3way-btn.js';

	export let name;

	const btn = new Push3WayBtn(name);

	onDestroy(() => {
		btn.destroy();
	});
</script>

<div class="dcs">
	<button
		class="up"
		on:click|preventDefault={() => btn.up()}
		class:active={$btn == UP}
	>
		U
	</button>
	<button
		class="down"
		on:click|preventDefault={() => btn.down()}
		class:active={$btn == DOWN}
	>
		D
	</button>
</div>

<style>
	.dcs {
		display: flex;
		flex-direction: column;
		gap: 5px;
	}

	button, .center {
		position: relative;
		display: block;
		width: 50px;
		height: 40px;
		background-color: var(--gray);
		outline: none;
		border: none;
		border-radius: 2px;
		font-size: 12px;
	}

	button.active, .center.active {
		background-color: var(--dark-gray);
	}

	.center {
		grid-area: 2 / 2 / 3 / 3;
	}

	.up {
		grid-area: 1 / 2 / 2 / 3;
	}

	.right {
		grid-area: 2 / 3 / 3 / 4;
	}

	.down {
		grid-area: 3 / 2 / 4 / 3;
	}

	.left {
		grid-area: 2 / 1 / 3 / 2;
	}
</style>