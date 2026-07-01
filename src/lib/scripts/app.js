import { writable } from "svelte/store";
import { getAppBridge } from "./app-bridge";

/** @type {import("svelte/store").Writable<AppData.Settings>} */
export let settings = writable({
  enableMouse: true,
  enableKeyboard: true,
  enableChapter: false,
  timerPaused: false,
});

export const chapterText = writable('');
export const chapterIndex = writable(0);

export const init = async () => {
  const appBridge = await getAppBridge();
  if (!appBridge) return;

  try {
    const s = await appBridge.getSettings();
    if (s) settings.set(s);
    chapterText.set(await appBridge.getChapterText());
    chapterIndex.set(await appBridge.getChapterIndex());
  }
  catch (e) {
    console.error(e);
  }

  appBridge.onChangeMouseEnable((/** @type {boolean} */ checked) => {
    settings.update((s) => {
      s.enableMouse = checked;
      return s;
    });
  });
  appBridge.onChangeKeyboardEnable((/** @type {boolean} */ checked) => {
    settings.update((s) => {
      s.enableKeyboard = checked;
      return s;
    });
  });
  appBridge.onChangeChapterEnable((/** @type {boolean} */ checked) => {
    settings.update((s) => {
      s.enableChapter = checked;
      return s;
    });
  });
  appBridge.onChangeTimerPaused((/** @type {boolean} */ checked) => {
    settings.update((s) => {
      s.timerPaused = checked;
      return s;
    });
  });

  appBridge.onChangeChapterText((/** @type {string} */ text) => {
    chapterText.set(text);
  });
  appBridge.onChangeChapterIndex((/** @type {number} */ index) => {
    chapterIndex.set(index);
  });
};

export const KEY_CONSTANTS = {
  shift: '⇧',
  control: '⌃',
  command: '⌘',
  option: '⌥',
  return: '⏎',
  delete: '⌫',
  tab: '⇥',
  // escape: '⎋',
  escape: 'esc',
  arrowRight: '→',
  arrowLeft: '←',
  arrowUp: '↑',
  arrowDown: '↓',
  slash: '/',
  backslash: '\\',
  minus: '-',
  comma: ',',
  period: '.',
  semicolon: ';',
  function: 'fn',
};

/**
 * TODO: 日本語レイアウトの場合おかしくなる問題があるが暫定対応
 * @type {Record<string, string>}
 */
export const KEY_NAME_TO_DISPLAY_TEXT_MAP = {
  'RightShift': KEY_CONSTANTS.shift,
  'Shift': KEY_CONSTANTS.shift,
  'RightControl': KEY_CONSTANTS.control,
  'Control': KEY_CONSTANTS.control,
  'RightOption': KEY_CONSTANTS.option,
  'Option': KEY_CONSTANTS.option,
  'RightCommand': KEY_CONSTANTS.command,
  'Command': KEY_CONSTANTS.command,
  'Escape': KEY_CONSTANTS.escape,
  'Tab': KEY_CONSTANTS.tab,
  'Return': KEY_CONSTANTS.return,
  'Delete': KEY_CONSTANTS.delete,
  'RightArrow': KEY_CONSTANTS.arrowRight,
  'LeftArrow': KEY_CONSTANTS.arrowLeft,
  'UpArrow': KEY_CONSTANTS.arrowUp,
  'DownArrow': KEY_CONSTANTS.arrowDown,
  'Slash': KEY_CONSTANTS.slash,
  // 'Backslash': KEY_CONSTANTS.backslash,
  'Minus': KEY_CONSTANTS.minus,
  'Comma': KEY_CONSTANTS.comma,
  'Period': KEY_CONSTANTS.period,
  'Semicolon': KEY_CONSTANTS.semicolon,
  'Function': KEY_CONSTANTS.function,
};

export const KEY_PRIORITIES = {
  [KEY_CONSTANTS.control]: 0,
  [KEY_CONSTANTS.option]: 1,
  [KEY_CONSTANTS.shift]: 2,
  [KEY_CONSTANTS.command]: 3,
};

export const MODIFIER_KEYS = new Set([
  KEY_CONSTANTS.shift,
  KEY_CONSTANTS.control,
  KEY_CONSTANTS.command,
  KEY_CONSTANTS.option,
  KEY_CONSTANTS.function,
]);

export const FUNCTION_KEYS = new Set([
  'F1', 'F2', 'F3', 'F4', 'F5', 'F6', 'F7', 'F8',
  'F9', 'F10', 'F11', 'F12', 'F13', 'F14', 'F15', 'F16',
  'F17', 'F18', 'F19', 'F20',
  KEY_CONSTANTS.escape,
  KEY_CONSTANTS.tab,
]);
