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
export const overlayVisible = writable(true);
export const inputMonitoringStatus = writable<InputMonitoringStatus | undefined>();

export const init = async () => {
  const appBridge = await getAppBridge();
  if (!appBridge) return;

  const updateSettings = (updater: (settings: AppData.Settings) => void) => {
    settings.update((current) => {
      const next = { ...current };
      updater(next);
      appBridge.setSettings(next).catch((error) => {
        console.error(error);
      });
      return next;
    });
  };

  await appBridge.onChangeOverlayVisible((visible: boolean) => {
    overlayVisible.set(visible);
  });
  await appBridge.onInputMonitoringStatus((status: InputMonitoringStatus) => {
    inputMonitoringStatus.set(status);
  });

  try {
    overlayVisible.set(await appBridge.getOverlayVisible());
    inputMonitoringStatus.set(await appBridge.getInputMonitoringStatus());
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
    updateSettings((s) => {
      s.enableMouse = checked;
    });
  });
  appBridge.onChangeKeyboardEnable((checked: boolean) => {
    updateSettings((s) => {
      s.enableKeyboard = checked;
    });
  });
  appBridge.onChangeChapterEnable((checked: boolean) => {
    updateSettings((s) => {
      s.enableChapter = checked;
    });
  });
  appBridge.onChangeTimerPaused((checked: boolean) => {
    updateSettings((s) => {
      s.timerPaused = checked;
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

export const KEYBOARD_LAYOUT_KEY_NAME_TO_DISPLAY_TEXT_MAP: Record<KeyboardLayout, Record<string, string>> = {
  unknown: {},
  us: {
    Backslash: KEY_CONSTANTS.backslash,
    BackQuote: KEY_CONSTANTS.backQuote,
  },
  jis: {
    Backslash: KEY_CONSTANTS.jisYen,
    JisYen: KEY_CONSTANTS.jisYen,
    JisUnderscore: KEY_CONSTANTS.jisUnderscore,
    Eisu: KEY_CONSTANTS.eisu,
    Kana: KEY_CONSTANTS.kana,
  },
};

export const toDisplayKeyName = (key = "", keyboardLayout: KeyboardLayout = "unknown"): string => {
  const layoutMap = KEYBOARD_LAYOUT_KEY_NAME_TO_DISPLAY_TEXT_MAP[keyboardLayout] || {};
  return layoutMap[key] || KEY_NAME_TO_DISPLAY_TEXT_MAP[key] || key;
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
