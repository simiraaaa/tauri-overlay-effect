<script>
	import { createEventDispatcher, onMount } from "svelte";

	/** @type {string[]} */
	export let keyNames = [];
	export let index = 0;
	export let keyListLength = 1;

	
	$: opacityRate = (index + 1) === keyListLength ? 1 : Math.max(0, 1 - (keyListLength - index) * 0.1) * 0.8;
	
	const dispatch = createEventDispatcher();
	const FADE_STATE_1 = 1;
	const FADE_STATE_2 = 2;
	const FADE_STATE_3 = 3;

	/** @type {HTMLDivElement | null} */
	let wrapperElement = null;

	const MIDDLE_OPACITY = 0.7;
	const MAX_OPACITY = 1;
	let state = 0;

	const ALL_DURATION = 2000;
	const START_DELAY = 200;

	const FADE_DURATION = 64;
	const MIDDLE_DURATION = 600;
	const DESTROY_DURATION = 320;

	let opacity = 0;

	onMount(() => {
		let count = 0;
		(function f() {
			requestAnimationFrame(() => {
				if (++count > 10) return ;
				if (wrapperElement?.offsetParent) {
					opacity = MAX_OPACITY;
				} else {
					f();
				}
			});
		})();
	});

	let duration = FADE_DURATION;
	let delay = 0;

	const onChangeState = () => {
		state++;
		if (state === FADE_STATE_1) {
			opacity = MIDDLE_OPACITY;
			duration = MIDDLE_DURATION;
			delay = START_DELAY;
		} else if (state === FADE_STATE_2) {
			opacity = 0;
			duration = DESTROY_DURATION;
			delay = ALL_DURATION - START_DELAY - DESTROY_DURATION - FADE_DURATION - MIDDLE_DURATION;
		} else if (state >= FADE_STATE_3) {
			dispatch('remove');
		}
	};

</script>

<div bind:this={wrapperElement} class="key-wrapper" style:--opacity={opacityRate}>
	<div
		class="key"
		on:transitionend={onChangeState}
		style:--transition-delay={delay}
		style:--transition-duration={duration}
		style:--opacity={opacity}
	>
		<div>
			{#each keyNames as key}
				<span class="key-item">{key}</span>
			{/each}
		</div>
	</div>
</div>

<style>

	.key-wrapper {
		opacity: var(--opacity);
	}

	.key {
		transition: calc(var(--transition-duration) * 1ms) opacity calc(var(--transition-delay) * 1ms);
		opacity: var(--opacity, 0);

		background: rgba(64, 64, 64, 0.8);
		padding: 8px 12px;
		display: flex;
		align-items: center;
		justify-content: center;

		word-break: keep-all;
		white-space: nowrap;
		
		border-radius: 8px;
		/* -webkit-text-stroke: 1px black; */
		box-shadow: 1px 1px 3px 1px rgba(0, 0, 0, 0.5);
		color: rgb(240, 240, 240);
		/* shadow を重ねることで縁取りしてる感じを出す */
		text-shadow: 0 0 2px rgba(0, 0, 0, 1),
			0 0 2px rgba(0, 0, 0, 1),
			0 0 2px rgba(0, 0, 0, 1),
			0 0 2px rgba(0, 0, 0, 1),
			0 0 2px rgba(0, 0, 0, 1),
			0 0 2px rgba(0, 0, 0, 1);
	}

	.key-item {
		display: inline-block;
		font-size: 36px;
		font-weight: 500;
		min-width: 0.8em;
		text-align: center;
		letter-spacing: 0.01em;
		font-feature-settings: "palt";
	}

</style>
