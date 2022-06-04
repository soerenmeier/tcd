<script>
	import { onDestroy } from 'svelte';
	import IcpDataControlSwitch, {
		CENTER, UP, RIGHT, DOWN, LEFT
	} from './../controls/icp-data-control-switch.js';

	const sw = new IcpDataControlSwitch;

	onDestroy(() => {
		sw.destroy();
	});
</script>

<div class="dcs">
	<button
		class="up"
		on:click|preventDefault={() => sw.up()}
		class:active={$sw == UP}
	>
		U
	</button>
	<button
		class="right"
		on:click|preventDefault={() => sw.right()}
		class:active={$sw == RIGHT}
	>
		SEQ
	</button>
	<button
		class="down"
		on:click|preventDefault={() => sw.down()}
		class:active={$sw == DOWN}
	>
		D
	</button>
	<button
		class="left"
		on:click|preventDefault={() => sw.left()}
		class:active={$sw == LEFT}
	>
		RTN
	</button>
	<!-- <span class="center" class:active={$sw == CENTER}></span> -->
</div>

<style>
	.dcs {
		display: grid;
		grid-template-columns: repeat(3, 40px);
		grid-gap: 5px;
	}

	button, .center {
		position: relative;
		display: block;
		width: 40px;
		height: 30px;
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