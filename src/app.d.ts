declare global {
	type Unlisten = () => void;
	type KeyboardLayout = 'unknown' | 'jis' | 'us';

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
		onGlobalKeyboard: (callback: (event: unknown, keyEvent: GlobalKeyEvent, down: GlobalKeyDownMap) => void) => Unlisten | Promise<Unlisten>;
		onLog: (callback: (...args: unknown[]) => void) => Unlisten | Promise<Unlisten>;
		onGlobalMouse: (callback: (event: unknown, mouseEvent: GlobalMouseEvent) => void) => Unlisten | Promise<Unlisten>;
		onInputMonitoringStatus: (callback: (status: InputMonitoringStatus) => void) => Unlisten | Promise<Unlisten>;
		onChangeOverlayVisible: (callback: (visible: boolean) => void) => Unlisten | Promise<Unlisten>;
		onChangeMouseEnable: (callback: (enable: boolean) => void) => Unlisten | Promise<Unlisten>;
		onChangeKeyboardEnable: (callback: (enable: boolean) => void) => Unlisten | Promise<Unlisten>;
		onChangeChapterEnable: (callback: (enable: boolean) => void) => Unlisten | Promise<Unlisten>;
		onChangeTimerPaused: (callback: (paused: boolean) => void) => Unlisten | Promise<Unlisten>;
		onChangeChapterText: (callback: (text: string) => void) => Unlisten | Promise<Unlisten>;
		onChangeChapterIndex: (callback: (index: number) => void) => Unlisten | Promise<Unlisten>;
		getOverlayVisible: () => Promise<boolean>;
		getInputMonitoringStatus: () => Promise<InputMonitoringStatus>;
		retryInputMonitoring: () => Promise<void>;
		setChapterInputPaused: (paused: boolean) => Promise<void>;
		getSettings: () => Promise<AppData.Settings>;
		setSettings: (settings: AppData.Settings) => Promise<void>;
		getChapterText: () => Promise<string>;
		setChapterText: (text: string) => Promise<void>;
		getChapterIndex: () => Promise<number>;
		setChapterIndex: (index: number) => Promise<{ last: number; index: number }>;
		addChapterIndex: (num: number) => Promise<{ last: number; index: number }>;
	};

	interface GlobalMouseEvent {
		type: 'down' | 'drag' | 'up' | string;
		x: number;
		y: number;
	}

	interface GlobalKeyDownMap {
		[key: string]: boolean | undefined;
	}

	interface GlobalKeyEvent {
		name?: string;
		state?: 'DOWN' | 'UP' | string;
		keyboardLayout?: KeyboardLayout;
		rawKey?: {
			name?: string;
			_nameRaw?: string;
		};
		[key: string]: unknown;
	}

	interface InputMonitoringStatus {
		state: 'starting' | 'active' | 'waiting' | 'failed' | 'disabled' | 'unsupported' | string;
		message: string;
		guidance?: string;
		canRetry: boolean;
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
