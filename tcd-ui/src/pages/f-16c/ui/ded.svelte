<script>
	import { onDestroy } from 'svelte';
	import { subscribe } from './../../../lib/controlsapi.js';

	let lines = [];
	let linesUnsub = [];

	for (let i = 0; i < 5; i++) {
		lines[i] = '.';
		linesUnsub[i] = () => {};

		subscribe('DED_LINE_' + (i + 1), outs => {
			if (!outs)
				return;

			let line = outs.string();
			line = line.replaceAll('a', '@');
			lines[i] = line;
			lines = lines;
			console.log('lines', lines);
		});
	}

	onDestroy(() => {
		while (true) {
			const unsub = linesUnsub.pop();
			if (!unsub)
				return;
			unsub();
		}
	});
</script>

<div id="ded">
	{#each lines as line}
		<p class="line">{line}</p>
	{/each}
</div>

<style>
	#ded {
		width: 210px;
		padding: 7px 0;
		font-family: "Falcon DED", serif;
		color: #3edd3e;
		background-color: var(--black);
	}

	.line {
		white-space: pre;
	}
</style>