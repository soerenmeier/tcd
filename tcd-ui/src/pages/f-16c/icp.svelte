<script>
	import { createEventDispatcher } from 'svelte';
	import Mfcd from './ui/mfcd.svelte';
	import BackBtn from './../../ui/back-btn.svelte';
	import Ded from './ui/ded.svelte';
	import IcpModeBtn from './ui/icp-mode-button.svelte';
	import IcpKeyBtn from './ui/icp-key-button.svelte';
	import IcpDataControlSwitch from './ui/icp-data-control-switch.svelte';
	import IcpPush3WayBtn from './ui/icp-push-3way-btn.svelte';

	const dispatch = createEventDispatcher();

	let cont = null;

	const padding = 20;

</script>

<div class="back-btn">
	<BackBtn on:click={() => dispatch('close')} />
</div>

<div id="icp">

	<div class="center">

		<Ded />

		<div class="modes">
			<IcpModeBtn name="ICP_COM1_BTN" label="COM 1" />
			<IcpModeBtn name="ICP_COM2_BTN" label="COM 2" />
			<IcpModeBtn name="ICP_IFF_BTN" label="IFF" />
			<IcpModeBtn name="ICP_LIST_BTN" label="LIST" />
			<IcpModeBtn name="ICP_AA_MODE_BTN" label="A-A" />
			<IcpModeBtn name="ICP_AG_MODE_BTN" label="A-G" />
		</div>

		<div class="keypad">
			<IcpKeyBtn name="ICP_BTN_1" number="1" label="T_ILS" />
			<IcpKeyBtn name="ICP_BTN_2" number="2" label="ALOW" small="N" />
			<IcpKeyBtn name="ICP_BTN_3" number="3" />
			<IcpKeyBtn name="ICP_RCL_BTN" label="RCL" />
			<IcpKeyBtn name="ICP_BTN_4" number="4" label="STPT" small="W" />
			<IcpKeyBtn name="ICP_BTN_5" number="5" label="CRUS" />
			<IcpKeyBtn name="ICP_BTN_6" number="6" label="TIME" small="E" />
			<IcpKeyBtn name="ICP_ENTR_BTN" label="ENTR" />
			<IcpKeyBtn name="ICP_BTN_7" number="7" label="MARK" />
			<IcpKeyBtn name="ICP_BTN_8" number="8" label="FIX" small="S" />
			<IcpKeyBtn name="ICP_BTN_9" number="9" label="A-CAL" />
			<IcpKeyBtn name="ICP_BTN_0" number="0" label="M-SEL" small="-" />
		</div>

		<div class="divers">
			<IcpPush3WayBtn name="ICP_DED_SW" />
			<IcpDataControlSwitch />
		</div>

	</div>

</div>

<style>
	#icp {
		display: flex;
		padding: 10px;
		justify-content: center;
	}

	.center {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.back-btn {
		position: absolute;
		top: 10px;
		left: 10px;
	}

	.modes {
		display: flex;
		justify-content: space-between;
	}

	.keypad {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		justify-items: center;
		grid-gap: 10px;
	}

	.divers {
		display: flex;
		justify-content: space-between;
	}
</style>