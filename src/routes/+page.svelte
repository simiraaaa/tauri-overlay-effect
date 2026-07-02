<script lang="ts">
	import Chapter from "$components/Chapter.svelte";
	import ChapterSettingPanel from "$components/ChapterSettingPanel.svelte";
	import Keyboard from "$components/Keyboard.svelte";
	import Mouse from "$components/Mouse.svelte";
	import { dev } from '$app/environment';
	import { getAppBridge } from "$lib/scripts/app-bridge";
	import { FUNCTION_KEYS, KEY_CONSTANTS, KEY_PRIORITIES, MODIFIER_KEYS, chapterIndex, chapterSettingVisible, chapterText, inputMonitoringStatus, overlayVisible, settings, toDisplayKeyName } from "$lib/scripts/app";
	import { onDestroy, onMount } from "svelte";

	type KeyParam = {
		id: Symbol;
		names: string[];
	};

	let logs = $state<unknown[]>([]);
	let keyParams = $state<KeyParam[]>([]);
	let pressedKeySet = $state(new Set<string>());
	let unlisteners = $state<(() => void)[]>([]);
	let keyboardLayout = $state<KeyboardLayout>('unknown');
	let pressedKeyIdleTimer: ReturnType<typeof setTimeout> | undefined;
	let ignoredStaleDownKeys = new Set<string>();
	let lastDownKeys = new Set<string>();

	const PRESSED_KEY_IDLE_RESET_MS = 2500;

	const log = (...args: unknown[]) => {
		logs.push(...args);
		logs = logs.slice(-60);
	};

	onMount(async () => {
		const appBridge = await getAppBridge();
		if (!appBridge) return;

		const logUnlisten = await appBridge.onLog(log);
		const keyboardUnlisten = await appBridge.onGlobalKeyboard(keydownHandler);
		if (typeof logUnlisten === 'function') unlisteners.push(logUnlisten);
		if (typeof keyboardUnlisten === 'function') unlisteners.push(keyboardUnlisten);
	});

	onDestroy(() => {
		clearPressedKeyIdleTimer();
		for (const unlisten of unlisteners) {
			unlisten();
		}
		unlisteners = [];
	});

	const keydownHandler = (_e: unknown, e: GlobalKeyEvent, down: GlobalKeyDownMap) => {
		if (!$overlayVisible) return;
		if (!$settings.enableKeyboard) return;
		if ($chapterSettingVisible) return;

		if (e.name?.startsWith('MOUSE')) {
			return;
		}

		keyboardLayout = e.keyboardLayout || keyboardLayout;
		schedulePressedKeyIdleReset();

		const rawKeyName = e.rawKey?.name || '';
		const display_key = toDisplayKeyName(rawKeyName, keyboardLayout);
		if (e.state === 'DOWN') {
			ignoredStaleDownKeys.delete(rawKeyName);
			syncPressedKeys(down, keyboardLayout);
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
				const display_keys = [...pressedKeySet].map((key) => toDisplayKeyName(key, keyboardLayout)).sort((a, b) => {
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
			ignoredStaleDownKeys.delete(rawKeyName);
			if (!syncPressedKeys(down, keyboardLayout)) {
				pressedKeySet.delete(display_key);
			}
		}
	};

	const syncPressedKeys = (down: GlobalKeyDownMap, layout: KeyboardLayout) => {
		if (!down || typeof down !== 'object') return false;

		lastDownKeys = new Set(
			Object.entries(down)
				.filter(([, pressed]) => pressed)
				.map(([key]) => key),
		);

		pressedKeySet = new Set(
			[...lastDownKeys]
				.filter((key) => !ignoredStaleDownKeys.has(key))
				.map((key) => toDisplayKeyName(key, layout)),
		);
		return true;
	};

	const clearPressedKeyIdleTimer = () => {
		if (pressedKeyIdleTimer) {
			clearTimeout(pressedKeyIdleTimer);
			pressedKeyIdleTimer = undefined;
		}
	};

	const schedulePressedKeyIdleReset = () => {
		clearPressedKeyIdleTimer();
		pressedKeyIdleTimer = setTimeout(() => {
			ignoredStaleDownKeys = new Set(
				[...lastDownKeys].filter((key) => !MODIFIER_KEYS.has(toDisplayKeyName(key, keyboardLayout))),
			);
			pressedKeySet = new Set();
			pressedKeyIdleTimer = undefined;
		}, PRESSED_KEY_IDLE_RESET_MS);
	};

	const pushKeys = (keys: string[] = []) => {
		keyParams.push({
			id: Symbol(),
			names: keys,
		});
		keyParams = keyParams.slice(-10);
	};

	const isDisplayable = (keys: string[] = []) => {
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

	const onRemoveKeyboard = (param: KeyParam) => {
		keyParams = keyParams.filter((p) => p.id !== param.id);
	};

	$effect(() => {
		if ($overlayVisible) return;
		clearPressedKeyIdleTimer();
		keyParams = [];
		pressedKeySet = new Set();
		ignoredStaleDownKeys = new Set();
		lastDownKeys = new Set();
	});

	let chapterLine = $derived(`${$chapterIndex + 1}. ` + $chapterText.split('\n')[$chapterIndex]);
	let showInputMonitoringNotice = $derived(
		$overlayVisible
		&& $inputMonitoringStatus
		&& !['starting', 'active'].includes($inputMonitoringStatus.state)
	);
</script>

<!-- <svelte:window on:keydown={onKeydown}></svelte:window> -->

<svelte:head>
	<title>Overlay effect</title>
	<meta name="description" content="Overlay effect" />
</svelte:head>

<section>
	{#if $chapterSettingVisible}
		<div class="chapter-setting-panel">
			<ChapterSettingPanel />
		</div>
	{/if}

	{#if $overlayVisible && $settings.enableChapter}
		<div class="chapter-container">
			<Chapter text={chapterLine}></Chapter>
			{#if $settings.timerPaused}
				<div class="paused">-- Paused --</div>
			{/if}
		</div>
	{/if}

	{#if dev}
		<div class="logs">
			{#each logs.slice().reverse() as log}
				<div>{log}</div>
			{/each}
		</div>
	{/if}
	{#if showInputMonitoringNotice && $inputMonitoringStatus}
		<div class="permission-notice">
			<div class="permission-title">入力監視が有効ではありません</div>
			<div>{$inputMonitoringStatus.message}</div>
			{#if $inputMonitoringStatus.guidance}
				<div>{$inputMonitoringStatus.guidance}</div>
			{/if}
			{#if $inputMonitoringStatus.canRetry}
				<div>許可後、トレイメニューの「入力監視を再試行」を選んでください。</div>
			{/if}
		</div>
	{/if}
	<div>
		{#if $overlayVisible && $settings.enableKeyboard}
			<div class="key-view-container">
				{#each keyParams as param, i (param.id)}
					<div class="key-item">
						<Keyboard
							keyNames={param.names}
							index={i}
							keyListLength={keyParams.length}
							onRemove={() => onRemoveKeyboard(param)}
						/>
					</div>
				{/each}
			</div>
		{/if}

	</div>
</section>
	{#if $overlayVisible && $settings.enableMouse}
		<Mouse log={log}></Mouse>
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

	.permission-notice {
		position: absolute;
		left: 24px;
		bottom: 24px;
		max-width: 460px;
		padding: 14px 16px;
		border-radius: 12px;
		background: rgba(24, 24, 24, 0.72);
		border: 1px solid rgba(255, 255, 255, 0.36);
		box-shadow: 0 12px 40px rgba(0, 0, 0, 0.25);
		color: white;
		font-size: 13px;
		line-height: 1.6;
		pointer-events: none;
	}

	.permission-title {
		font-size: 14px;
		font-weight: 700;
		margin-bottom: 4px;
	}

	.chapter-setting-panel {
		position: absolute;
		inset: 0;
		z-index: 100;
		width: min(420px, calc(100vw - 48px));
		height: min(560px, calc(100vh - 48px));
		margin: auto;
		border-radius: 12px;
		overflow: hidden;
		box-shadow: 0 16px 60px rgba(0, 0, 0, 0.35);
		pointer-events: auto;
	}

</style>
