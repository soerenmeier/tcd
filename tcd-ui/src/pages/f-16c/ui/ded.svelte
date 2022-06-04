<script>
	import { onMount, onDestroy } from 'svelte';
	import { subscribe } from './../../../lib/controlsapi.js';
	import DedFont, {
		width as fontWidth,
		height as fontHeight
	} from './dedfont.js';
	import Context2d from 'fire/dom/context2d.js';

	const gapX = 2;
	const gapY = 3;
	const lineLen = 24;
	const linesLen = 5;

	const charWidth = Math.floor(fontWidth / 2.5);
	const charHeight = Math.floor(fontHeight / 2.5);

	const width = lineLen * charWidth + (lineLen - 1) * gapX;
	const height = linesLen * charHeight + (linesLen - 1) * gapY;

	let lines = [];
	let linesUnsub = [];
	let canvas = null;
	let drop = false;

	for (let i = 0; i < linesLen; i++) {
		lines[i] = [];

		linesUnsub[i] = subscribe('DED_LINE_' + (i + 1), outs => {
			if (!outs)
				return;

			let line = outs.string() ?? '';
			lines[i] = line;
		});
	}

	let ctx = null;

	const font = new DedFont('ded_font.png');
	const fontInv = new DedFont('ded_font_inv.png');

	function render() {
		if (drop)
			return;

		ctx.clearAll();
		ctx.ctx.globalCompositeOperation = 'source-over';
		ctx.fillStyle = '#000';
		ctx.fillRect(0, 0, ctx.width, ctx.height);

		lines.forEach((line, y) => {
			const rY = y * charHeight + (y * gapY);

			let x = 0;
			for (let i = 0; i < line.length; (i++, x++)) {
				const rX = x * charWidth + (x * gapX);
				const char = line[i];
				// escape character to inverse
				if (char === '\r') {
					i++;
					fontInv.draw(ctx, line[i], rX, rY, charWidth, charHeight);
				} else {
					font.draw(ctx, char, rX, rY, charWidth, charHeight);
				}
			}
		});

		ctx.ctx.globalCompositeOperation = 'multiply';
		ctx.fillStyle = '#34f700';
		ctx.fillRect(0, 0, ctx.width, ctx.height);

		requestAnimationFrame(render);
	}

	onMount(() => {
		ctx = new Context2d(canvas);
		ctx.updateSize(width, height);

		requestAnimationFrame(render);
	});

	onDestroy(() => {
		drop = true;

		while (true) {
			const unsub = linesUnsub.pop();
			if (!unsub)
				return;
			unsub();
		}
	});
</script>

<div id="ded">
	<canvas bind:this={canvas}></canvas>
</div>