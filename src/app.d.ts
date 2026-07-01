declare global {
	namespace App {
		// interface Error {}
		// interface Locals {}
		// interface PageData {}
		// interface Platform {}
	}
	
	var electron: AppBridge | undefined;
	var appBridge: AppBridge | undefined;
	var isDev: boolean;

	type AppBridge = {
		onGlobalKeyboard: (callback: (...args: any[]) => void) => unknown;
		onLog: (callback: (...args: any[]) => void) => unknown;
		onGlobalMouse: (callback: (...args: any[]) => void) => unknown;
		onChangeMouseEnable: (callback: (enable: boolean) => void) => unknown;
		onChangeKeyboardEnable: (callback: (enable: boolean) => void) => unknown;
		onChangeChapterEnable: (callback: (enable: boolean) => void) => unknown;
		onChangeTimerPaused: (callback: (paused: boolean) => void) => unknown;
		onChangeChapterText: (callback: (text: string) => void) => unknown;
		onChangeChapterIndex: (callback: (index: number) => void) => unknown;
		getSettings: () => Promise<AppData.Settings>;
		getChapterText: () => Promise<string>;
		setChapterText: (text: string) => Promise<void>;
		getChapterIndex: () => Promise<number>;
		setChapterIndex: (index: number) => Promise<{ last: number; index: number }>;
		addChapterIndex: (num: number) => Promise<{ last: number; index: number }>;
	};

	interface GlobalKeyDownMap {
		[key: string]: boolean | undefined;
	}

	interface GlobalKeyEvent {
		name?: string;
		state?: 'DOWN' | 'UP' | string;
		rawKey?: {
			name?: string;
			_nameRaw?: string;
		};
		[key: string]: unknown;
	}

	namespace AppData {
		type Settings = {
			enableMouse: boolean;
			enableKeyboard: boolean;
			enableChapter: boolean;
			timerPaused: boolean;
		};
	}
}

export {};
