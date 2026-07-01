<script>
	import Chapter from "$components/Chapter.svelte";
	import Keyboard from "$components/Keyboard.svelte";
	import Mouse from "$components/Mouse.svelte";
	import { getAppBridge } from "$lib/scripts/app-bridge";
	import { FUNCTION_KEYS, KEY_CONSTANTS, KEY_NAME_TO_DISPLAY_TEXT_MAP, KEY_PRIORITIES, MODIFIER_KEYS, chapterIndex, chapterText, settings } from "$lib/scripts/app";
	import { onMount } from "svelte";
	
	/** @type {string[]} */
	let logs = [];

	/**
	 * @typedef {{ id: Symbol; names: string[] }} KeyParam
	 */

	/** @type {KeyParam[]} */
	let keyParams = [];

	/** @type {Set<string>} */
	let pressedKeySet = new Set();

	const log = (/** @type {any[]} */ ...args) => {
		logs.push(...args);
		logs = logs.slice(-60);
	};

	onMount(async () => {
		const appBridge = await getAppBridge();
		appBridge.onLog(log);
		appBridge.onGlobalKeyboard(keydownHandler);
	});

	const keydownHandler = (_e = {}, /** @type {GlobalKeyEvent} **/ e, /** @type {GlobalKeyDownMap} */ down) => {
		if (!$settings.enableKeyboard) return ;
		
		// log(`_raw: ${e._raw}, vKey: ${e.vKey}, name: ${e.name}, scanCode: ${e.scanCode}, rawKey._nameRaw: ${e.rawKey?._nameRaw}, rawKey.name: ${e.rawKey?.name}`);
		// skip mouse
		if (e.name?.startsWith('MOUSE')) {
			return;
		}
		const display_key = toDisplayKeyName(e.rawKey?.name);
		if (e.state === 'DOWN'){
			// pushKeys(e.rawKey?.name);
			pressedKeySet.add(display_key);
			let key_display_threshold = 2;
			if (pressedKeySet.has(KEY_CONSTANTS.shift)) {
				key_display_threshold++;
			}
			for (const key of pressedKeySet) {
				if (FUNCTION_KEYS.has(key)) {
					key_display_threshold = 1;
				}
			}
			if (pressedKeySet.size >= key_display_threshold) {
				const display_keys = [...pressedKeySet].map(toDisplayKeyName).sort((a, b) => {
					let ap = Infinity;
					let bp = Infinity;
					if (a in KEY_PRIORITIES) {
						ap = KEY_PRIORITIES[a];
					}
					if (b in KEY_PRIORITIES) {
						bp = KEY_PRIORITIES[b];
					}
					return ap - bp;
				});
				if (isDisplayable(display_keys)) {
					pushKeys(display_keys);
				}
			}
		} else if (e.state === 'UP') {
			pressedKeySet.delete(display_key);
		}
	};

	/** @type {(keys: string[]) => void} */
	const pushKeys = (keys = []) => {
		keyParams.push({
			id: Symbol(),
			names: keys,
		});
		keyParams = keyParams.slice(-10);
		keyParams = keyParams;
	};

	/** @type {(keys: string[]) => boolean} */
	const isDisplayable = (keys = []) => {
		// shift を除く
		let has_modifier_key = false;
		let has_other_key = false;
		let has_function_key = false;
		keys.forEach((key) => {
			if (FUNCTION_KEYS.has(key)) {
				has_function_key = true;
			}
			if (MODIFIER_KEYS.has(key)) {
				if (key !== KEY_CONSTANTS.shift) {
					has_modifier_key = true;
				}
			} else {
				has_other_key = true;
			}
		});

		return has_function_key || (has_modifier_key && has_other_key);
	};

	const toDisplayKeyName = (key = '') => {
		// 暫定対応
		const text = KEY_NAME_TO_DISPLAY_TEXT_MAP[key]
		if (text) {
			return text;
		}
		return key;
	};

	/** @type {(param: KeyParam) => void} */
	const onRemoveKeyboard = (param) => {
		keyParams = keyParams.filter((p) => p.id !== param.id);
	};

	$: chapterLine = `${$chapterIndex + 1}. ` + $chapterText.split('\n')[$chapterIndex];

</script>

<!-- <svelte:window on:keydown={onKeydown}></svelte:window> -->

<svelte:head>
	<title>Overlay effect</title>
	<meta name="description" content="Overlay effect" />
</svelte:head>

<section>
	<!-- チャプター -->
	{#if $settings.enableChapter}
		<div class="chapter-container">
			<Chapter text="{chapterLine}"></Chapter>
			{#if $settings.timerPaused}
				<div class="paused">-- Paused --</div>
			{/if}
		</div>
	{/if}

	<!-- svelte-ignore missing-declaration -->
	{#if isDev}
		<div class="logs">
			{#each logs.slice().reverse() as log}
				<div>{log}</div>
			{/each}
		</div>
	{/if}
	<div>
		{#if $settings.enableKeyboard}
			<!-- 一番最後のキーが真ん中に表示されるようにする -->
			<div class="key-view-container">
				{#each keyParams as param, i (param.id)}
					<div class="key-item">
						<Keyboard 
							keyNames={param.names}
							index={i}
							keyListLength={keyParams.length}
							on:remove={() => onRemoveKeyboard(param)}
						/>
					</div>
				{/each}
			</div>
		{/if}
		
	</div>
</section>
{#if $settings.enableMouse}
	<Mouse {log}></Mouse>
{/if}


<style>
	section {
		position: relative;
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		flex-shrink: 0;
		flex-grow: 1;
	}
	.logs {
		padding: 16px;
		background-color: white;
		position: absolute;
		top: 0;
		left: 0;
		font-size: 12px;
		color: black;
		opacity: 0.5;
	}

	.key-view-container {
		pointer-events: none;
		display: flex;
		width: 0;
		height: 0;
		position: absolute;
		left: 0;
		right: 0;
		bottom: 33%;
		margin: auto;
		flex-direction: column;
		justify-content: end;
		align-items: center;
		flex-shrink: 0;
		flex-grow: 1;
	}

	.key-item {
		margin-bottom: 16px;
		display: flex;
		justify-content: center;
		align-items: center;
		flex-shrink: 0;
		flex-grow: 1;
	}

	.chapter-container {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		margin: auto;
		display: flex;
		justify-content: center;
		align-items: center;
	}

	.paused {
		font-size: 24px;
		color: darkblue;
		padding: 16px;
		border-radius: 8px;
		background-color: lightgray;
		opacity: 0.5;
		position: absolute;
		margin: auto;
	}

</style>
