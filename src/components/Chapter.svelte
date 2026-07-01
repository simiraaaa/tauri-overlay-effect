<script>
	import { run } from 'svelte/legacy';

    import { chapterIndex } from "$lib/scripts/app";
	import { getAppBridge } from "$lib/scripts/app-bridge";
	import { debounce } from "$lib/scripts/util";
	import { onMount } from "svelte";

	/**
	 * @typedef {Object} Props
	 * @property {string} [text]
	 */

	/** @type {Props} */
	let { text = '' } = $props();
	let appendingText = $state('');
	/** @type {AppBridge | undefined} */
	let appBridge;

	let hovered = $state(false);

	let continuousHoverCount = $state(0);
	// 3 往復ぐらいしたらページ切り替え
	const TRIGGER_HOVER_COUNT = 6;

	// Tauri のクリック透過は Electron の forward 相当が未確認のため、ホバー送りは後続フェーズで再設計する。
	const onMouseEnter = () => {
		hovered = true;
	};

	const onMouseLeave = () => {
		hovered = false;
	};

	run(() => {
		text;
		continuousHoverCount = 0;
		hovered = false;
		appendingText = '';
	});

	onMount(async () => {
		appBridge = await getAppBridge();
	});

	/**
	 * @param {MouseEvent} e
	 */
	const onMouseEnterChapter = async (e) => {
		continuousHoverCount++;
		if (continuousHoverCount >= TRIGGER_HOVER_COUNT) {
			// 画面左なら 前へ
			let num = -1;

			// 画面右なら 次へ
			if (e.clientX > window.innerWidth / 2) {
				num = 1;
			}
			const current = $chapterIndex;
			const bridge = appBridge || await getAppBridge();
			const {index, last} = await bridge.addChapterIndex(num);
			
			if (current === index) {
				if (index <= 0) {
					appendingText = '最初のチャプターです';
				}
				else if (index >= last) {
					appendingText = '最後のチャプターです';
				}
			}
			continuousHoverCount = 0;
		}
		else {
			resetContinuousHoverCount();
		}
	};

	const resetContinuousHoverCount = debounce(() => {
		continuousHoverCount = 0;
	}, 300);
	
</script>

{#key text}
	<div class="wrapper" class:hovered role="button" aria-label="chapter container" tabindex="-1" onmouseenter={onMouseEnter} onmouseleave={onMouseLeave}>
		<div class="chapter" role="button" aria-label="chapter" tabindex="-1" onmouseenter={onMouseEnterChapter}>
			{text}
			{#if appendingText}
				<div class="appending-text">({appendingText})</div>
			{/if}
		</div>
	</div>
{/key}


<style>
	.wrapper {
		width: 60%;
		min-height: 30vh;
		display: flex;
		justify-content: center;
		align-items: center;

		pointer-events: none;

		animation: wrapper-animation 0.5s ease forwards;
		animation-delay: 3s;
	}

	.chapter {
		background-color: #fff;
		color: #111;
		padding: 40px 48px;
		font-size: 40px;
		font-weight: bold;
		border-radius: 4px;
		width: 100%;
		word-break: break-all;

		box-shadow: 0 0 16px rgba(0, 0, 0, 0.5);
		animation: chapter-animation 0.5s ease forwards;
		animation-delay: 3s;
	}

	.appending-text {
		font-size: 0.8em;
		color: orange;
	}

	.wrapper.hovered .chapter {
		transition: opacity 128ms ease-in-out;
		opacity: 0.1;
	}

	@keyframes wrapper-animation {
		to {
			transform: translateY(calc(50vh - 50%)) scale(0.7);
			pointer-events: auto;
			opacity: 0.5;
		}
	}

	@keyframes chapter-animation {
		to {
			box-shadow: none;
			padding: 12px 24px;
			width: auto;
		}
	}

</style>
