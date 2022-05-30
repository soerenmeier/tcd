<script>
	import { onDestroy } from 'svelte';
	import { subscribe } from './../lib/mfdsapi.js';
	import Context2d from 'fire/dom/context2d.js';

	let ctx = null;
	let unsub = () => {};

	function newCanvas(el) {
		ctx = new Context2d(el);
		ctx.updateSize();

		ctx.fillRect(0, 0, 100, 100);

		subscribe(frame => {
			if (!frame)
				return;

			ctx.clearAll();
			ctx.drawImage(frame, 0, 0);
		});
	}

	onDestroy(() => {
		unsub();
	});
</script>

<div class="full-mfds full-size">
	<canvas use:newCanvas></canvas>
</div>

<style>
	canvas {
		width: 100%;
		height: 100%;
	}
</style>