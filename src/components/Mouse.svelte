<script>
	import { getAppBridge } from "$lib/scripts/app-bridge";
	import { onDestroy, onMount } from "svelte";
	import { writable } from "svelte/store";

	let x = writable(0);
	let y = writable(0);
	let _visible = false;
	let _time = 0;
	/** @type {NodeJS.Timeout | null} */
	let _timeout_id = null;
	let visible = writable(false);
	// 最低でも 3 フレーム分は表示する
	const MIN_VISIBLE_TIME = 1000 / 60 * 3;

	const onGlobalMouse = (/** @type {any} */ _e,/** @type {any} */ mouse) => {
		if (mouse.type === 'down' || mouse.type === 'drag') {
			x.set(Math.floor(mouse.x));
			y.set(Math.floor(mouse.y));
			if (!_visible) {
				_time = performance.now();
				visible.set(_visible = true);
			}
			// clog?.(JSON.stringify(mouse, null, '  '));
		} else {
			if (performance.now() - _time > MIN_VISIBLE_TIME) {
				if (_timeout_id) {
					clearTimeout(_timeout_id);
					_timeout_id = null;
				}
				visible.set(_visible = false);
			} else {
				if (!_timeout_id) {
					_timeout_id = setTimeout(() => {
						visible.set(_visible = false);
						_timeout_id = null;
					}, MIN_VISIBLE_TIME);
				}
			}
		}
	};

	let clog = (/** @type {any[]} */ ...args) => console.log(...args);
	export let log = (/** @type {any[]} */ ...args) => console.log(...args);
	$: {clog = log;}

	const size = 50;
	const stroke = 2;
	/** @type {(() => void) | null} */
	let unlisten = null;

	onMount(async () => {
		const appBridge = await getAppBridge();
		const cleanup = await appBridge.onGlobalMouse(onGlobalMouse);
		if (typeof cleanup === 'function') {
			unlisten = /** @type {() => void} */ (cleanup);
		}
	});

	onDestroy(() => {
		if (typeof unlisten === 'function') {
			unlisten();
		}
	});

</script>


<div class="overlay"
	style:--size={size}px 
	style:--size-wrap={size + stroke}px
	style:--stroke={stroke}px
	style:--stroke-wrap={stroke * 2}px
	style:--x={$x}px
	style:--y={$y}px
	style:--visibility={$visible ? 'visible' : 'hidden'}
>
	<div class="mouse"></div>
	<div class="mouse-inner"></div>
</div>


<style>
	.overlay {
		pointer-events: none;
		position: absolute;
		top: 0;
		left: 0;
		width: 100vw;
		height: 100vh;

		/* background: rgba(0, 0, 0, 0.1); */
	}

	.mouse {
		position: absolute;
		left: var(--x);
		top: var(--y);
		width: var(--size-wrap);
		height: var(--size-wrap);
		transform: translate3d(-50%, -50%, 1px);
		border-radius: 50%;
		/* background: rgba(255, 255, 255, 0.5); */
		border: var(--stroke-wrap) solid white;
		visibility: var(--visibility);
	}

	.mouse-inner {
		position: absolute;
		display: block;
		left: var(--x);
		top: var(--y);
		width: var(--size);
		height: var(--size);
		transform: translate3d(-50%, -50%, 1px);
		border-radius: 50%;
		border: var(--stroke) solid black;
		visibility: var(--visibility);
	}


</style>
