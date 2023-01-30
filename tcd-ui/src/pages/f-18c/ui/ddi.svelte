<script>
	import { onMount, onDestroy } from 'svelte';
	import DdiButton from './ddi-button.svelte';
	import Context2d from 'fire/dom/context2d.js';
	import { timeout } from 'fire/util.js';
	import { newMfdWebrtc } from './../../../lib/mfdsapi.js';
	import { newError } from './../../../lib/errors.js';

	export let name;
	export let size;
	export let kind;

	let cont = null;
	let video = null;

	let destroyed = false;
	let unsubscribe = () => {};

	const displayPadding = 2 * 10 + 2 * 40;

	onMount(async () => {
		// calculate size
		const s = size();
		cont.style.width = s + 'px';
		cont.style.height = s + 'px';

		video.width = s - displayPadding;
		video.height = s - displayPadding;

		// now create a mfd stream
		try {
			const pc = await newMfdWebrtc(kind, video);
			unsubscribe = () => { pc.close() };
			if (destroyed)
				pc.close();
		} catch (e) {
			console.log('failed to create webrtc', e);
			newError('failed to create webrtc');
		}
	});

	onDestroy(() => {
		destroyed = true;
		unsubscribe();
	});
</script>

<div class="mfcd" bind:this={cont}>
	<div class="btns top">
		<DdiButton name={name + '_06'} />
		<DdiButton name={name + '_07'} />
		<DdiButton name={name + '_08'} />
		<DdiButton name={name + '_09'} />
		<DdiButton name={name + '_10'} />
	</div>
	<div class="btns right">
		<DdiButton name={name + '_11'} />
		<DdiButton name={name + '_12'} />
		<DdiButton name={name + '_13'} />
		<DdiButton name={name + '_14'} />
		<DdiButton name={name + '_15'} />
	</div>
	<div class="btns bottom">
		<DdiButton name={name + '_20'} />
		<DdiButton name={name + '_19'} />
		<DdiButton name={name + '_18'} />
		<DdiButton name={name + '_17'} />
		<DdiButton name={name + '_16'} />
	</div>
	<div class="btns left">
		<DdiButton name={name + '_05'} />
		<DdiButton name={name + '_04'} />
		<DdiButton name={name + '_03'} />
		<DdiButton name={name + '_02'} />
		<DdiButton name={name + '_01'} />
	</div>

	<video bind:this={video}></video>
</div>

<style>
	.mfcd {
		display: grid;
		grid-gap: 10px;
		grid-template-columns: 40px 1fr 40px;
		grid-template-rows: 40px 1fr 40px;
	}

	.btns {
		display: flex;
		gap: 10px;
		justify-content: space-evenly;
	}

	.left, .right {
		flex-direction: column;
	}

	.top {
		grid-area: 1 / 2 / 2 / 3;
	}

	.right {
		grid-area: 2 / 3 / 3 / 4;
	}

	.left {
		grid-area: 2 / 1 / 3 / 2;
	}

	.bottom {
		grid-area: 3 / 2 / 4 / 3;
	}

	video {
		display: block;
		width: 100%;
		height: 100%;
		grid-area: 2 / 2 / 3 / 3;
	}
</style>