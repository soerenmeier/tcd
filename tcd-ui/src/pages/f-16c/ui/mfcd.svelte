<script>
	import { onMount, onDestroy } from 'svelte';
	import MfcdButton from './mfcd-button.svelte';
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

		// const ctx = new Context2d(canvas);
		// ctx.updateSize(s - displayPadding, s - displayPadding);

		// ctx.fillRect(0, 0, ctx.width, ctx.height);

		// unsubscribe = subscribe(kind, frame => {
		// 	if (!frame)
		// 		return;

		// 	ctx.clearAll();
		// 	ctx.drawImage(frame, 0, 0, ctx.width, ctx.height);
		// });
	});

	onDestroy(() => {
		destroyed = true;
		unsubscribe();
	});
</script>

<div class="mfcd" bind:this={cont}>
	<div class="btns top">
		<MfcdButton name={name + '_1'} />
		<MfcdButton name={name + '_2'} />
		<MfcdButton name={name + '_3'} />
		<MfcdButton name={name + '_4'} />
		<MfcdButton name={name + '_5'} />
	</div>
	<div class="btns right">
		<MfcdButton name={name + '_6'} />
		<MfcdButton name={name + '_7'} />
		<MfcdButton name={name + '_8'} />
		<MfcdButton name={name + '_9'} />
		<MfcdButton name={name + '_10'} />
	</div>
	<div class="btns bottom">
		<MfcdButton name={name + '_15'} />
		<MfcdButton name={name + '_14'} />
		<MfcdButton name={name + '_13'} />
		<MfcdButton name={name + '_12'} />
		<MfcdButton name={name + '_11'} />
	</div>
	<div class="btns left">
		<MfcdButton name={name + '_20'} />
		<MfcdButton name={name + '_19'} />
		<MfcdButton name={name + '_18'} />
		<MfcdButton name={name + '_17'} />
		<MfcdButton name={name + '_16'} />
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