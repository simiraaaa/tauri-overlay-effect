import type { UnlistenFn } from '@tauri-apps/api/event';

const defaultSettings: AppData.Settings = {
  enableMouse: true,
  enableKeyboard: true,
  enableChapter: false,
  timerPaused: false,
};

type PayloadCallback<T> = (payload: T) => void;

const noopUnlisten: UnlistenFn = () => {};

const noopBridge: AppBridge = {
  onGlobalKeyboard: () => noopUnlisten,
  onLog: () => noopUnlisten,
  onGlobalMouse: () => noopUnlisten,
  onChangeMouseEnable: () => noopUnlisten,
  onChangeKeyboardEnable: () => noopUnlisten,
  onChangeChapterEnable: () => noopUnlisten,
  onChangeTimerPaused: () => noopUnlisten,
  onChangeChapterText: () => noopUnlisten,
  onChangeChapterIndex: () => noopUnlisten,
  getSettings: async () => defaultSettings,
  getChapterText: async () => "",
  setChapterText: async () => {},
  getChapterIndex: async () => 0,
  setChapterIndex: async (index = 0) => ({ index, last: 0 }),
  addChapterIndex: async (num = 0) => ({ index: 0, last: 0, num }),
};

let bridgePromise: Promise<AppBridge | undefined> | undefined;

export const getAppBridge = async (): Promise<AppBridge | undefined> => {
  if (bridgePromise) return bridgePromise;

  bridgePromise = createAppBridge();
  globalThis.appBridge = await bridgePromise;
  return globalThis.appBridge;
};

const createAppBridge = async (): Promise<AppBridge> => {
  if (globalThis.electron) {
    return globalThis.electron;
  }

  if (isTauriRuntime()) {
    return createTauriBridge();
  }

  return noopBridge;
};

const isTauriRuntime = (): boolean => {
  return '__TAURI_INTERNALS__' in globalThis || '__TAURI__' in globalThis;
};

const createTauriBridge = async (): Promise<AppBridge> => {
  const [{ invoke }, { listen }] = await Promise.all([
    import('@tauri-apps/api/core'),
    import('@tauri-apps/api/event'),
  ]);

  const listenPayload = async <T>(eventName: string, callback: PayloadCallback<T>): Promise<UnlistenFn> => {
    return listen<T>(eventName, (event) => callback(event.payload));
  };

  const bridge: AppBridge = {
    onGlobalKeyboard: (callback) => listenPayload('global-key', (payload) => {
      if (Array.isArray(payload)) {
        const [rawEvent, down] = payload as [GlobalKeyEvent, GlobalKeyDownMap];
        callback({}, rawEvent, down);
        return;
      }

      const eventPayload = payload as { event?: GlobalKeyEvent; down?: GlobalKeyDownMap };
      callback({}, eventPayload?.event ?? eventPayload, eventPayload?.down ?? {});
    }),
    onLog: (callback) => listenPayload('log', (payload) => {
      if (globalThis.isDev) callback(payload);
    }),
    onGlobalMouse: (callback) => listenPayload('global-mouse', (payload) => {
      callback({}, payload as GlobalMouseEvent);
    }),
    onChangeMouseEnable: (callback) => listenPayload<boolean>('change-mouse-enable', callback),
    onChangeKeyboardEnable: (callback) => listenPayload<boolean>('change-keyboard-enable', callback),
    onChangeChapterEnable: (callback) => listenPayload<boolean>('change-chapter-enable', callback),
    onChangeTimerPaused: (callback) => listenPayload<boolean>('change-timer-paused', callback),
    onChangeChapterText: (callback) => listenPayload<string>('change-chapter-text', callback),
    onChangeChapterIndex: (callback) => listenPayload<number>('change-chapter-index', callback),
    getSettings: () => invoke('get_settings'),
    getChapterText: () => invoke('get_chapter_text'),
    setChapterText: (text = "") => invoke('set_chapter_text', { text }),
    getChapterIndex: () => invoke('get_chapter_index'),
    setChapterIndex: (index = 0) => invoke('set_chapter_index', { index }),
    addChapterIndex: (num = 0) => invoke('add_chapter_index', { num }),
  };

  return bridge;
};
