import { writable } from "svelte/store";
import { getAppBridge } from "./app-bridge";

export const settings = writable<AppData.Settings>({
  enableMouse: true,
  enableKeyboard: true,
  enableChapter: false,
  timerPaused: false,
});

export const chapterText = writable("");
export const chapterIndex = writable(0);

export const init = async () => {
  const appBridge = await getAppBridge();
  if (!appBridge) return;

  try {
    const s = await appBridge.getSettings();
    if (s) {
      settings.set(s);
    }
    chapterText.set(await appBridge.getChapterText());
    chapterIndex.set(await appBridge.getChapterIndex());
  } catch (e) {
    console.error(e);
  }

  appBridge.onChangeMouseEnable((checked: boolean) => {
    settings.update((s) => {
      s.enableMouse = checked;
      return s;
    });
  });
  appBridge.onChangeKeyboardEnable((checked: boolean) => {
    settings.update((s) => {
      s.enableKeyboard = checked;
      return s;
    });
  });
  appBridge.onChangeChapterEnable((checked: boolean) => {
    settings.update((s) => {
      s.enableChapter = checked;
      return s;
    });
  });
  appBridge.onChangeTimerPaused((checked: boolean) => {
    settings.update((s) => {
      s.timerPaused = checked;
      return s;
    });
  });

  appBridge.onChangeChapterText((text: string) => {
    chapterText.set(text);
  });
  appBridge.onChangeChapterIndex((index: number) => {
    chapterIndex.set(index);
  });
};

export const KEY_CONSTANTS: Record<string, string> = {
  shift: "⇧",
  control: "⌃",
  command: "⌘",
  option: "⌥",
  return: "⏎",
  delete: "⌫",
  tab: "⇥",
  // escape: '⎋',
  escape: "esc",
  arrowRight: "→",
  arrowLeft: "←",
  arrowUp: "↑",
  arrowDown: "↓",
  slash: "/",
  backslash: "\\",
  minus: "-",
  equal: "=",
  comma: ",",
  period: ".",
  semicolon: ";",
  quote: "'",
  backQuote: "`",
  space: "space",
  jisYen: "¥",
  jisUnderscore: "_",
  eisu: "英数",
  kana: "かな",
  function: "fn",
};

export const KEY_NAME_TO_DISPLAY_TEXT_MAP: Record<string, string> = {
  RightShift: KEY_CONSTANTS.shift,
  Shift: KEY_CONSTANTS.shift,
  RightControl: KEY_CONSTANTS.control,
  Control: KEY_CONSTANTS.control,
  RightOption: KEY_CONSTANTS.option,
  Option: KEY_CONSTANTS.option,
  RightCommand: KEY_CONSTANTS.command,
  Command: KEY_CONSTANTS.command,
  Escape: KEY_CONSTANTS.escape,
  Tab: KEY_CONSTANTS.tab,
  Return: KEY_CONSTANTS.return,
  Delete: KEY_CONSTANTS.delete,
  RightArrow: KEY_CONSTANTS.arrowRight,
  LeftArrow: KEY_CONSTANTS.arrowLeft,
  UpArrow: KEY_CONSTANTS.arrowUp,
  DownArrow: KEY_CONSTANTS.arrowDown,
  LeftBracket: "[",
  RightBracket: "]",
  Slash: KEY_CONSTANTS.slash,
  Backslash: KEY_CONSTANTS.backslash,
  Minus: KEY_CONSTANTS.minus,
  Equal: KEY_CONSTANTS.equal,
  Plus: "+",
  Multiply: "*",
  Comma: KEY_CONSTANTS.comma,
  Period: KEY_CONSTANTS.period,
  Semicolon: KEY_CONSTANTS.semicolon,
  Quote: KEY_CONSTANTS.quote,
  BackQuote: KEY_CONSTANTS.backQuote,
  Space: KEY_CONSTANTS.space,
  JisYen: KEY_CONSTANTS.jisYen,
  JisUnderscore: KEY_CONSTANTS.jisUnderscore,
  Eisu: KEY_CONSTANTS.eisu,
  Kana: KEY_CONSTANTS.kana,
  Function: KEY_CONSTANTS.function,
};

export const KEY_PRIORITIES: Record<string, number> = {
  [KEY_CONSTANTS.control]: 0,
  [KEY_CONSTANTS.option]: 1,
  [KEY_CONSTANTS.shift]: 2,
  [KEY_CONSTANTS.command]: 3,
};

export const MODIFIER_KEYS: Set<string> = new Set([
  KEY_CONSTANTS.shift,
  KEY_CONSTANTS.control,
  KEY_CONSTANTS.command,
  KEY_CONSTANTS.option,
  KEY_CONSTANTS.function,
]);

export const FUNCTION_KEYS: Set<string> = new Set([
  "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8",
  "F9", "F10", "F11", "F12", "F13", "F14", "F15", "F16",
  "F17", "F18", "F19", "F20",
  KEY_CONSTANTS.escape,
  KEY_CONSTANTS.tab,
]);
