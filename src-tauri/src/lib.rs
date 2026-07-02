use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::panic::{self, AssertUnwindSafe};

#[cfg(target_os = "macos")]
use rdev::{listen, Button, Event, EventType, Key};
#[cfg(target_os = "macos")]
use objc2_app_kit::{
    NSScreenSaverWindowLevel, NSWindow, NSWindowCollectionBehavior,
};
use tauri::{Emitter, Manager};
use tauri::{PhysicalPosition, PhysicalSize};

#[derive(Default)]
struct AppState {
    storage_path: Option<PathBuf>,
    data: PersistedState,
}

#[derive(Clone, Deserialize, Serialize)]
struct Settings {
    #[serde(default = "default_enabled", rename = "enableMouse")]
    enable_mouse: bool,
    #[serde(default = "default_enabled", rename = "enableKeyboard")]
    enable_keyboard: bool,
    #[serde(default, rename = "enableChapter")]
    enable_chapter: bool,
    #[serde(default, rename = "timerPaused")]
    timer_paused: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enable_mouse: true,
            enable_keyboard: true,
            enable_chapter: false,
            timer_paused: false,
        }
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
struct PersistedState {
    #[serde(default)]
    settings: Settings,
    #[serde(default, rename = "chapterText")]
    chapter_text: String,
    #[serde(default, rename = "chapterIndex")]
    chapter_index: usize,
}

fn default_enabled() -> bool {
    true
}

#[derive(Serialize)]
struct ChapterIndexResult {
    index: usize,
    last: usize,
}

#[derive(Clone, Serialize)]
struct MouseEvent {
    position: &'static str,
    #[serde(rename = "type")]
    event_type: &'static str,
    x: i32,
    y: i32,
}

#[derive(Clone, Serialize)]
struct RawKey {
    #[serde(rename = "name")]
    name: Option<String>,
    #[serde(rename = "_nameRaw")]
    name_raw: Option<String>,
}

#[derive(Clone, Serialize)]
struct KeyEvent {
    name: String,
    state: &'static str,
    #[serde(rename = "keyboardLayout")]
    keyboard_layout: KeyboardLayout,
    #[serde(rename = "rawKey")]
    raw_key: RawKey,
}

#[derive(Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum KeyboardLayout {
    Unknown,
    Jis,
    Us,
}

#[tauri::command]
fn get_settings(state: tauri::State<'_, Mutex<AppState>>) -> Settings {
    state
        .lock()
        .map(|state| state.data.settings.clone())
        .unwrap_or_default()
}

#[tauri::command]
fn set_settings(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<AppState>>,
    settings: Settings,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|error| error.to_string())?;
    let mut next = state.data.clone();
    next.settings = settings.clone();
    save_persisted_state(&state.storage_path, &next)?;
    state.data = next;
    emit_settings_changed(&app, &settings);
    Ok(())
}

#[tauri::command]
fn get_chapter_text(state: tauri::State<'_, Mutex<AppState>>) -> String {
    state
        .lock()
        .map(|state| state.data.chapter_text.clone())
        .unwrap_or_default()
}

#[tauri::command]
fn set_chapter_text(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<AppState>>,
    text: String,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|error| error.to_string())?;
    let mut next = state.data.clone();
    next.chapter_text = text.clone();

    let last = last_chapter_index(&next.chapter_text);
    if next.chapter_index > last {
        next.chapter_index = last;
    }
    save_persisted_state(&state.storage_path, &next)?;
    state.data = next;

    let _ = app.emit("change-chapter-text", text);
    let _ = app.emit("change-chapter-index", state.data.chapter_index);
    Ok(())
}

#[tauri::command]
fn get_chapter_index(state: tauri::State<'_, Mutex<AppState>>) -> usize {
    state
        .lock()
        .map(|state| state.data.chapter_index)
        .unwrap_or_default()
}

#[tauri::command]
fn set_chapter_index(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<AppState>>,
    index: usize,
) -> Result<ChapterIndexResult, String> {
    let mut state = state.lock().map_err(|error| error.to_string())?;
    let mut next = state.data.clone();
    let result = set_chapter_index_inner(&mut next, index);
    save_persisted_state(&state.storage_path, &next)?;
    state.data = next;
    let _ = app.emit("change-chapter-index", result.index);
    Ok(result)
}

#[tauri::command]
fn add_chapter_index(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<AppState>>,
    num: isize,
) -> Result<ChapterIndexResult, String> {
    let mut state = state.lock().map_err(|error| error.to_string())?;
    let current = state.data.chapter_index as isize;
    let next = current.saturating_add(num).max(0) as usize;
    let mut next_state = state.data.clone();
    let result = set_chapter_index_inner(&mut next_state, next);
    save_persisted_state(&state.storage_path, &next_state)?;
    state.data = next_state;
    let _ = app.emit("change-chapter-index", result.index);
    Ok(result)
}

fn set_chapter_index_inner(state: &mut PersistedState, index: usize) -> ChapterIndexResult {
    let last = last_chapter_index(&state.chapter_text);
    state.chapter_index = index.min(last);

    ChapterIndexResult {
        index: state.chapter_index,
        last,
    }
}

fn last_chapter_index(text: &str) -> usize {
    text.lines().count().saturating_sub(1)
}

fn initialize_persisted_state(app: &tauri::AppHandle) -> Result<(), String> {
    let storage_path = persisted_state_path(app)?;
    let data = match load_persisted_state(&storage_path) {
        Ok(data) => data,
        Err(error) => {
            let quarantined_path = quarantine_persisted_state(&storage_path)?;
            eprintln!(
                "Failed to read persisted app state: {error}. Moved the broken file to {}",
                quarantined_path.display()
            );
            PersistedState::default()
        }
    };
    let state = app.state::<Mutex<AppState>>();
    let mut state = state.lock().map_err(|error| error.to_string())?;
    state.storage_path = Some(storage_path);
    state.data = normalize_persisted_state(data);
    Ok(())
}

fn persisted_state_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let directory = app.path().app_config_dir().map_err(|error| error.to_string())?;
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(directory.join("state.json"))
}

fn load_persisted_state(path: &PathBuf) -> Result<PersistedState, String> {
    if !path.exists() {
        return Ok(PersistedState::default());
    }

    let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str::<PersistedState>(&contents).map_err(|error| error.to_string())
}

fn quarantine_persisted_state(path: &PathBuf) -> Result<PathBuf, String> {
    if !path.exists() {
        return Ok(path.clone());
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_secs();
    let quarantined_path = path.with_extension(format!("json.invalid-{timestamp}"));
    fs::rename(path, &quarantined_path).map_err(|error| error.to_string())?;
    Ok(quarantined_path)
}

fn normalize_persisted_state(mut state: PersistedState) -> PersistedState {
    let last = last_chapter_index(&state.chapter_text);
    if state.chapter_index > last {
        state.chapter_index = last;
    }
    state
}

fn save_persisted_state(path: &Option<PathBuf>, data: &PersistedState) -> Result<(), String> {
    let Some(path) = path else {
        return Ok(());
    };

    if let Some(directory) = path.parent() {
        fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    }

    let contents = serde_json::to_string_pretty(data).map_err(|error| error.to_string())?;
    let temporary_path = path.with_extension("json.tmp");
    fs::write(&temporary_path, contents).map_err(|error| error.to_string())?;
    fs::rename(&temporary_path, path).map_err(|error| error.to_string())?;
    Ok(())
}

fn emit_settings_changed(app: &tauri::AppHandle, settings: &Settings) {
    let _ = app.emit("change-mouse-enable", settings.enable_mouse);
    let _ = app.emit("change-keyboard-enable", settings.enable_keyboard);
    let _ = app.emit("change-chapter-enable", settings.enable_chapter);
    let _ = app.emit("change-timer-paused", settings.timer_paused);
}

fn emit_global_mouse_event(app: &tauri::AppHandle, event: MouseEvent) {
    if let Some(window) = app.get_webview_window("main") {
        if window.emit("global-mouse", &event).is_ok() {
            return;
        }
    }

    if let Err(error) = app.emit("global-mouse", event) {
        eprintln!("Failed to emit global mouse event: {error:?}");
    }
}

fn emit_global_key_event(app: &tauri::AppHandle, event: KeyEvent, down: &HashMap<String, bool>) {
    let payload = (event, down);
    if let Some(window) = app.get_webview_window("main") {
        if window.emit("global-key", &payload).is_ok() {
            return;
        }
    }

    if let Err(error) = app.emit("global-key", payload) {
        eprintln!("Failed to emit global keyboard event: {error:?}");
    }
}

fn emit_log(app: &tauri::AppHandle, message: &str) {
    let _ = app.emit("log", message.to_string());
}

#[cfg(target_os = "macos")]
fn normalize_global_mouse_position(
    app: &tauri::AppHandle,
    raw_x: i32,
    raw_y: i32,
    last_position: &Arc<Mutex<(i32, i32)>>,
) -> (i32, i32) {
    let Some(window) = app.get_webview_window("main") else {
        return (raw_x, raw_y);
    };

    let monitor = window
        .monitor_from_point(raw_x as f64, raw_y as f64)
        .ok()
        .flatten()
        .or_else(|| window.current_monitor().ok().flatten());

    let Some(monitor) = monitor else {
        return (raw_x, raw_y);
    };

    let monitor_position = monitor.position();
    let monitor_size = monitor.size();
    let monitor_left = monitor_position.x as f64;
    let monitor_top = monitor_position.y as f64;
    let monitor_width = monitor_size.width as f64;
    let monitor_height = monitor_size.height as f64;

    let scaled_x = raw_x as f64 - monitor_left;
    let scaled_y_from_top = raw_y as f64 - monitor_top;
    let scaled_y_from_bottom = (monitor_top + monitor_height) - raw_y as f64;

    let top_candidate = (scaled_x.round() as i32, scaled_y_from_top.round() as i32);
    let bottom_candidate = (scaled_x.round() as i32, scaled_y_from_bottom.round() as i32);

    let mut x = bottom_candidate.0;
    let mut y = bottom_candidate.1;

    if let Ok((last_x, last_y)) = last_position.lock().map(|position| *position) {
        let top_delta = (top_candidate.0 - last_x).abs() + (top_candidate.1 - last_y).abs();
        let bottom_delta = (bottom_candidate.0 - last_x).abs() + (bottom_candidate.1 - last_y).abs();

        if top_delta < bottom_delta {
            x = top_candidate.0;
            y = top_candidate.1;
        }
    }

    let max_x = monitor_width.round() as i32;
    let max_y = monitor_height.round() as i32;

    if x < 0 {
        x = 0;
    }
    if x > max_x {
        x = max_x;
    }
    if y < 0 {
        y = 0;
    }
    if y > max_y {
        y = max_y;
    }

    let normalized = (x, y);
    if let Ok(mut last) = last_position.lock() {
        *last = normalized;
    }

    normalized
}

#[cfg(target_os = "macos")]
fn spawn_global_input_events(app: tauri::AppHandle, event_seen: Arc<AtomicBool>) -> Result<(), String> {
    let is_button_down = Arc::new(AtomicBool::new(false));
    let cursor_position = Arc::new(Mutex::new((0i32, 0i32)));
    let normalized_position = Arc::new(Mutex::new((0i32, 0i32)));
    let pressed_keys = Arc::new(Mutex::new(HashMap::<String, bool>::new()));
    let active_key_names = Arc::new(Mutex::new(HashMap::<Key, String>::new()));
    let detected_keyboard_layout = Arc::new(Mutex::new(KeyboardLayout::Unknown));
    let app_for_events = app.clone();

    let is_button_down_for_events = Arc::clone(&is_button_down);
    let cursor_position_for_events = Arc::clone(&cursor_position);
    let normalized_position_for_events = Arc::clone(&normalized_position);
    let pressed_keys_for_events = Arc::clone(&pressed_keys);
    let active_key_names_for_events = Arc::clone(&active_key_names);
    let detected_keyboard_layout_for_events = Arc::clone(&detected_keyboard_layout);
    let app_for_normalize_events = app_for_events.clone();

    let listener = move |event: Event| {
        if let Err(error) = panic::catch_unwind(AssertUnwindSafe(|| {
            event_seen.store(true, Ordering::SeqCst);

            match event.event_type {
                EventType::MouseMove { x, y } => {
                    let x = x as i32;
                    let y = y as i32;
                    let (x, y) = normalize_global_mouse_position(
                        &app_for_normalize_events,
                        x,
                        y,
                        &normalized_position_for_events,
                    );
                    if let Ok(mut position) = cursor_position_for_events.lock() {
                        *position = (x, y);
                    }

                    if is_button_down_for_events.load(Ordering::Relaxed) {
                        emit_global_mouse_event(
                            &app_for_events,
                            MouseEvent {
                                position: "left",
                                event_type: "drag",
                                x,
                                y,
                            },
                        );
                    }
                }
                EventType::ButtonPress(button) => {
                    let (x, y) = cursor_position_for_events.lock().map(|position| *position).unwrap_or((0, 0));

                    is_button_down.store(true, Ordering::Relaxed);

                    let position = match button {
                        Button::Left => "left",
                        Button::Right => "right",
                        _ => "other",
                    };

                    emit_global_mouse_event(
                        &app_for_events,
                        MouseEvent {
                            position,
                            event_type: "down",
                            x,
                            y,
                        },
                    );
                }
                EventType::ButtonRelease(button) => {
                    let (x, y) = cursor_position_for_events.lock().map(|position| *position).unwrap_or((0, 0));
                    is_button_down.store(false, Ordering::Relaxed);

                    let position = match button {
                        Button::Left => "left",
                        Button::Right => "right",
                        _ => "other",
                    };

                    emit_global_mouse_event(
                        &app_for_events,
                        MouseEvent {
                            position,
                            event_type: "up",
                            x,
                            y,
                        },
                    );
                }
                EventType::KeyPress(key) => {
                    let keyboard_layout = update_keyboard_layout(
                        &detected_keyboard_layout_for_events,
                        key,
                        event.name.as_ref(),
                    );
                    let (name, name_raw) = key_name_for_event(event.name.as_ref(), key);
                    let _ = active_key_names_for_events.lock().map(|mut active| {
                        active.insert(key, name.clone());
                    });
                    let _ = pressed_keys_for_events.lock().map(|mut down| {
                        down.insert(name.clone(), true);
                    });
                    let raw_name = name_raw.unwrap_or_else(|| name.clone());

                    emit_key_if_state_changed(
                        &app_for_events,
                        &pressed_keys_for_events,
                        KeyEvent {
                            name: name.clone(),
                            state: "DOWN",
                            keyboard_layout,
                            raw_key: RawKey {
                                name: Some(name.clone()),
                                name_raw: Some(raw_name),
                            },
                        },
                    );
                }
                EventType::KeyRelease(key) => {
                    let keyboard_layout = update_keyboard_layout(
                        &detected_keyboard_layout_for_events,
                        key,
                        event.name.as_ref(),
                    );
                    let (fallback_name, name_raw) = key_name_for_event(event.name.as_ref(), key);
                    let name = active_key_names_for_events
                        .lock()
                        .ok()
                        .and_then(|mut active| active.remove(&key))
                        .unwrap_or(fallback_name);
                    let _ = pressed_keys_for_events.lock().map(|mut down| {
                        down.remove(&name);
                    });

                    emit_key_if_state_changed(
                        &app_for_events,
                        &pressed_keys_for_events,
                        KeyEvent {
                            name: name.clone(),
                            state: "UP",
                            keyboard_layout,
                            raw_key: RawKey {
                                name: Some(name.clone()),
                                name_raw: Some(name_raw.unwrap_or(name.clone())),
                            },
                        },
                    );
                }
                _ => {}
            }
        })) {
            eprintln!("Panic in global mouse callback: {:?}", error);
        }
    };

    match panic::catch_unwind(AssertUnwindSafe(|| listen(listener))) {
        Ok(result) => result.map_err(|error| format!("{error:?}")),
        Err(error) => Err(format!("Panic in global mouse listener: {:?}", error)),
    }
}

#[cfg(not(target_os = "macos"))]
fn spawn_global_input_events(_app: tauri::AppHandle, _event_seen: Arc<AtomicBool>) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "macos")]
fn emit_key_if_state_changed(
    app: &tauri::AppHandle,
    pressed_keys: &Arc<Mutex<HashMap<String, bool>>>,
    event: KeyEvent,
) {
    let down = match pressed_keys.lock() {
        Ok(state) => state.clone(),
        Err(_) => HashMap::new(),
    };

    emit_global_key_event(app, event, &down);
}

#[cfg(target_os = "macos")]
fn update_keyboard_layout(
    detected_keyboard_layout: &Arc<Mutex<KeyboardLayout>>,
    key: Key,
    event_name: Option<&String>,
) -> KeyboardLayout {
    let inferred = infer_keyboard_layout(key, event_name);

    let Ok(mut current) = detected_keyboard_layout.lock() else {
        return inferred.unwrap_or(KeyboardLayout::Unknown);
    };

    match (*current, inferred) {
        (KeyboardLayout::Jis, _) => KeyboardLayout::Jis,
        (_, Some(KeyboardLayout::Jis)) => {
            *current = KeyboardLayout::Jis;
            KeyboardLayout::Jis
        }
        (KeyboardLayout::Unknown, Some(layout)) => {
            *current = layout;
            layout
        }
        _ => *current,
    }
}

#[cfg(target_os = "macos")]
fn infer_keyboard_layout(key: Key, event_name: Option<&String>) -> Option<KeyboardLayout> {
    if matches!(key, Key::Unknown(93) | Key::Unknown(94) | Key::Unknown(102) | Key::Unknown(104)) {
        return Some(KeyboardLayout::Jis);
    }

    if let Some(name) = event_name {
        if name == "¥" || name == "￥" {
            return Some(KeyboardLayout::Jis);
        }
        if matches!(key, Key::BackSlash) && name == "\\" {
            return Some(KeyboardLayout::Us);
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn normalize_key_raw_name(name: &str) -> String {
    match name {
        "Alt" => "Option".to_string(),
        "Meta" => "Command".to_string(),
        "Super" => "Command".to_string(),
        _ => {
            if name.len() == 1 {
                name.to_ascii_uppercase()
            } else {
                name.to_string()
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn key_name_for_event(event_name: Option<&String>, key: Key) -> (String, Option<String>) {
    if let Some(name) = stable_key_name_from_physical_key(key) {
        return (name.clone(), Some(format!("{key:?}")));
    }

    let raw = event_name
        .filter(|name| is_printable_key_name(name))
        .cloned();
    if let Some(name) = &raw {
        let normalized = normalize_key_raw_name(name);
        return (normalized, Some(name.clone()));
    }

    let fallback = format!("{key:?}");
    (fallback, raw)
}

#[cfg(target_os = "macos")]
fn stable_key_name_from_physical_key(key: Key) -> Option<String> {
    let name = match key {
        Key::ShiftLeft => "Shift".to_string(),
        Key::ShiftRight => "RightShift".to_string(),
        Key::ControlLeft => "Control".to_string(),
        Key::ControlRight => "RightControl".to_string(),
        Key::Alt => "Option".to_string(),
        Key::AltGr => "RightOption".to_string(),
        Key::MetaLeft => "Command".to_string(),
        Key::MetaRight => "RightCommand".to_string(),
        Key::Backspace => "Delete".to_string(),
        Key::Return => "Return".to_string(),
        Key::CapsLock => "CapsLock".to_string(),
        Key::Delete => "Delete".to_string(),
        Key::Tab => "Tab".to_string(),
        Key::Escape => "Escape".to_string(),
        Key::Home => "Home".to_string(),
        Key::End => "End".to_string(),
        Key::PageUp => "PageUp".to_string(),
        Key::PageDown => "PageDown".to_string(),
        Key::UpArrow => "UpArrow".to_string(),
        Key::DownArrow => "DownArrow".to_string(),
        Key::LeftArrow => "LeftArrow".to_string(),
        Key::RightArrow => "RightArrow".to_string(),
        Key::Function => "Function".to_string(),
        Key::F1 => "F1".to_string(),
        Key::F2 => "F2".to_string(),
        Key::F3 => "F3".to_string(),
        Key::F4 => "F4".to_string(),
        Key::F5 => "F5".to_string(),
        Key::F6 => "F6".to_string(),
        Key::F7 => "F7".to_string(),
        Key::F8 => "F8".to_string(),
        Key::F9 => "F9".to_string(),
        Key::F10 => "F10".to_string(),
        Key::F11 => "F11".to_string(),
        Key::F12 => "F12".to_string(),
        Key::Space => "Space".to_string(),
        Key::KpReturn => "Return".to_string(),
        Key::Num1 => "1".to_string(),
        Key::Num2 => "2".to_string(),
        Key::Num3 => "3".to_string(),
        Key::Num4 => "4".to_string(),
        Key::Num5 => "5".to_string(),
        Key::Num6 => "6".to_string(),
        Key::Num7 => "7".to_string(),
        Key::Num8 => "8".to_string(),
        Key::Num9 => "9".to_string(),
        Key::Num0 => "0".to_string(),
        Key::NumLock => "NumLock".to_string(),
        Key::KpMinus => "Minus".to_string(),
        Key::KpPlus => "Plus".to_string(),
        Key::KpMultiply => "Multiply".to_string(),
        Key::KpDivide => "Slash".to_string(),
        Key::Kp0 => "0".to_string(),
        Key::Kp1 => "1".to_string(),
        Key::Kp2 => "2".to_string(),
        Key::Kp3 => "3".to_string(),
        Key::Kp4 => "4".to_string(),
        Key::Kp5 => "5".to_string(),
        Key::Kp6 => "6".to_string(),
        Key::Kp7 => "7".to_string(),
        Key::Kp8 => "8".to_string(),
        Key::Kp9 => "9".to_string(),
        Key::KpDelete => "Delete".to_string(),
        Key::PrintScreen => "PrintScreen".to_string(),
        Key::ScrollLock => "ScrollLock".to_string(),
        Key::Pause => "Pause".to_string(),
        Key::Insert => "Insert".to_string(),
        Key::Unknown(93) => "JisYen".to_string(),
        Key::Unknown(94) => "JisUnderscore".to_string(),
        Key::Unknown(102) => "Eisu".to_string(),
        Key::Unknown(104) => "Kana".to_string(),
        Key::Unknown(_) => return None,
        _ => {
            let raw = format!("{key:?}");
            if let Some(rest) = raw.strip_prefix("Key") {
                rest.to_string()
            } else {
                return None;
            }
        }
    };

    Some(name)
}

#[cfg(target_os = "macos")]
fn is_printable_key_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|character| {
            !character.is_control()
                && !matches!(character as u32, 0xE000..=0xF8FF)
        })
}

#[cfg(not(target_os = "macos"))]
fn apply_overlay_window_bounds(window: &tauri::WebviewWindow) {
    let monitor = window.current_monitor().ok().flatten().or_else(|| window.primary_monitor().ok().flatten());

    if let Some(monitor) = monitor {
        let position = *monitor.position();
        let size = *monitor.size();
        let _ = window.set_position(PhysicalPosition::new(position.x, position.y));
        let _ = window.set_size(PhysicalSize::new(size.width, size.height));
    }
}

#[cfg(target_os = "macos")]
fn apply_overlay_window_bounds(window: &tauri::WebviewWindow) {
    if let Some(monitor) = window.current_monitor().ok().flatten().or_else(|| window.primary_monitor().ok().flatten()) {
        let position = *monitor.position();
        let size = *monitor.size();
        let _ = window.set_position(PhysicalPosition::new(position.x, position.y));
        let _ = window.set_size(PhysicalSize::new(size.width, size.height));
    }
}

#[cfg(target_os = "macos")]
fn apply_macos_overlay_window_level(window: &tauri::WebviewWindow) {
    let _ = window.set_always_on_top(true);
    let _ = window.set_visible_on_all_workspaces(true);

    let ns_window = match window.ns_window() {
        Ok(handle) if !handle.is_null() => handle as *mut NSWindow,
        _ => return,
    };

    let ns_window: &NSWindow = unsafe { &*ns_window };
    set_overlay_ns_window_level(ns_window);

    // メニューバー領域まで含めて画面全体を覆う。
    // AppKit はメニューバーより低いレベルのウィンドウを constrainFrameRect で
    // メニューバー高さ分だけ下へ押し下げる。レベルを上げた「後」に画面全体の
    // フレームを再設定することで、この押し下げを回避しウィンドウ原点を画面最上部に揃える。
    // NSScreen の frame はポイント単位なので Retina スケールにも追従する。
    if let Some(screen) = ns_window.screen() {
        ns_window.setFrame_display(screen.frame(), true);
    }
}

// ウィンドウレベルとコレクションビヘイビアだけを適用する（フレームには触れない）。
// メニューバー(24)より高い NSScreenSaverWindowLevel(1000) に置くことで前面へ出す。
#[cfg(target_os = "macos")]
fn set_overlay_ns_window_level(ns_window: &NSWindow) {
    ns_window.setLevel(NSScreenSaverWindowLevel);
    ns_window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::Stationary,
    );
}

// Tauri は webview 初期化時に config の alwaysOnTop を再適用し、
// ウィンドウレベルをメニューバーより低い値へ戻してしまう。
// 表示後にレベルだけを再主張して、メニューバー前面を維持する。
#[cfg(target_os = "macos")]
fn reassert_overlay_window_level(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if let Ok(handle) = window.ns_window() {
            if !handle.is_null() {
                let ns_window: &NSWindow = unsafe { &*(handle as *mut NSWindow) };
                set_overlay_ns_window_level(ns_window);
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn reassert_overlay_window_level(_app: &tauri::AppHandle) {}

#[cfg(not(target_os = "macos"))]
fn apply_macos_overlay_window_level(_window: &tauri::WebviewWindow) {}

pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            get_chapter_text,
            set_chapter_text,
            get_chapter_index,
            set_chapter_index,
            add_chapter_index
        ])
        .setup(|app| {
            if let Err(error) = panic::catch_unwind(AssertUnwindSafe(|| {
                #[cfg(target_os = "macos")]
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);

                let setup_app = app.handle().clone();
                if let Err(error) = initialize_persisted_state(&setup_app) {
                    let message = format!("Failed to load persisted app state: {error}");
                    eprintln!("{message}");
                    emit_log(&setup_app, &message);
                }

                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_always_on_top(true);
                    let _ = window.set_ignore_cursor_events(true);
                    let _ = window.set_shadow(false);
                    let _ = window.set_decorations(false);
                    apply_overlay_window_bounds(&window);
                    apply_macos_overlay_window_level(&window);

                    // フォーカス/移動/リサイズ時に Tauri がレベルを戻すことがあるため、
                    // それらのイベントごとにメニューバー前面レベルを再主張する。
                    let level_guard_app = app.handle().clone();
                    window.on_window_event(move |event| match event {
                        tauri::WindowEvent::Focused(_)
                        | tauri::WindowEvent::Moved(_)
                        | tauri::WindowEvent::Resized(_) => {
                            reassert_overlay_window_level(&level_guard_app);
                        }
                        _ => {}
                    });
                }

                // 起動直後、Tauri が config の alwaysOnTop を webview 初期化時に
                // 再適用してレベルをメニューバーより下へ戻す。そのタイミングを確実に
                // 上書きするため、最初の数秒はレベルを複数回再主張する。
                #[cfg(target_os = "macos")]
                {
                    let reassert_app = app.handle().clone();
                    thread::spawn(move || {
                        for delay_ms in [80u64, 150, 300, 500, 800, 1200, 2000] {
                            thread::sleep(std::time::Duration::from_millis(delay_ms));
                            let step_app = reassert_app.clone();
                            let _ = reassert_app.run_on_main_thread(move || {
                                reassert_overlay_window_level(&step_app);
                            });
                        }
                    });
                }

                let listener_app = app.handle().clone();
                let event_seen = Arc::new(AtomicBool::new(false));
                let event_seen_for_listener = Arc::clone(&event_seen);

                if std::env::var("OVERLAY_DISABLE_MOUSE_LISTENER").ok().as_deref() != Some("1") {
                    let _watchdog_app = app.handle().clone();
                    thread::spawn(move || {
                        if let Err(error) = spawn_global_input_events(listener_app, event_seen_for_listener) {
                            let message = format!(
                                "Failed to start global input monitoring: {error}. Please allow Accessibility for this app in System Settings > Privacy & Security > Accessibility."
                            );
                            eprintln!("{message}");
                            emit_log(&_watchdog_app, &message);
                        }
                    });
                } else {
                    eprintln!("Global mouse listener disabled by OVERLAY_DISABLE_MOUSE_LISTENER=1");
                }
            })) {
                eprintln!("Panic in app setup: {:?}", error);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run tauri application");
}
