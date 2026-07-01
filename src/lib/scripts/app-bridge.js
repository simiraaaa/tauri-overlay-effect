/** @typedef {(payload: any) => void} PayloadCallback */

/** @type {AppData.Settings} */
const defaultSettings = {
  enableMouse: true,
  enableKeyboard: true,
  enableChapter: false,
  timerPaused: false,
};

/** @returns {void} */
const noopUnlisten = () => {};

/** @type {AppBridge} */
const noopBridge = {
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
  getChapterText: async () => '',
  setChapterText: async () => {},
  getChapterIndex: async () => 0,
  setChapterIndex: async (index = 0) => ({ index, last: 0 }),
  addChapterIndex: async () => ({ index: 0, last: 0 }),
};

/** @type {Promise<AppBridge> | undefined} */
let bridgePromise;

/** @returns {Promise<AppBridge>} */
export const getAppBridge = async () => {
  if (bridgePromise) return bridgePromise;

  bridgePromise = createAppBridge();
  globalThis.appBridge = await bridgePromise;
  return globalThis.appBridge;
};

/** @returns {Promise<AppBridge>} */
const createAppBridge = async () => {
  if (globalThis.electron) {
    return globalThis.electron;
  }

  if (isTauriRuntime()) {
    return createTauriBridge();
  }

  return noopBridge;
};

const isTauriRuntime = () => {
  return '__TAURI_INTERNALS__' in globalThis || '__TAURI__' in globalThis;
};

/** @returns {Promise<AppBridge>} */
const createTauriBridge = async () => {
  const [{ invoke }, { listen }] = await Promise.all([
    import('@tauri-apps/api/core'),
    import('@tauri-apps/api/event'),
  ]);

  /**
   * @param {string} eventName
   * @param {PayloadCallback} callback
   */
  const listenPayload = async (eventName, callback) => {
    return listen(eventName, (event) => callback(event.payload));
  };

  /** @type {AppBridge} */
  const bridge = {
    onGlobalKeyboard: (callback) => listenPayload('global-key', (payload) => {
      if (Array.isArray(payload)) {
        callback({}, ...payload);
        return;
      }
      callback({}, payload?.event ?? payload, payload?.down ?? {});
    }),
    onLog: (callback) => listenPayload('log', (payload) => {
      if (isDev) callback(payload);
    }),
    onGlobalMouse: (callback) => listenPayload('global-mouse', (payload) => {
      callback({}, payload);
    }),
    onChangeMouseEnable: (callback) => listenPayload('change-mouse-enable', callback),
    onChangeKeyboardEnable: (callback) => listenPayload('change-keyboard-enable', callback),
    onChangeChapterEnable: (callback) => listenPayload('change-chapter-enable', callback),
    onChangeTimerPaused: (callback) => listenPayload('change-timer-paused', callback),
    onChangeChapterText: (callback) => listenPayload('change-chapter-text', callback),
    onChangeChapterIndex: (callback) => listenPayload('change-chapter-index', callback),
    getSettings: () => invoke('get_settings'),
    getChapterText: () => invoke('get_chapter_text'),
    setChapterText: (text = '') => invoke('set_chapter_text', { text }),
    getChapterIndex: () => invoke('get_chapter_index'),
    setChapterIndex: (index = 0) => invoke('set_chapter_index', { index }),
    addChapterIndex: (num = 0) => invoke('add_chapter_index', { num }),
  };

  return bridge;
};
