use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::panic::{self, AssertUnwindSafe};

#[cfg(target_os = "macos")]
use rdev::{listen, Button, Event, EventType};
use tauri::{Emitter, Manager};
use tauri::{PhysicalPosition, PhysicalSize};

#[derive(Default)]
struct AppState {
    chapter_text: String,
    chapter_index: usize,
}

#[derive(Serialize)]
struct Settings {
    #[serde(rename = "enableMouse")]
    enable_mouse: bool,
    #[serde(rename = "enableKeyboard")]
    enable_keyboard: bool,
    #[serde(rename = "enableChapter")]
    enable_chapter: bool,
    #[serde(rename = "timerPaused")]
    timer_paused: bool,
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

#[tauri::command]
fn get_settings() -> Settings {
    Settings {
        enable_mouse: true,
        enable_keyboard: true,
        enable_chapter: false,
        timer_paused: false,
    }
}

#[tauri::command]
fn get_chapter_text(state: tauri::State<'_, Mutex<AppState>>) -> String {
    state.lock().map(|state| state.chapter_text.clone()).unwrap_or_default()
}

#[tauri::command]
fn set_chapter_text(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<AppState>>,
    text: String,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|error| error.to_string())?;
    state.chapter_text = text.clone();

    let last = last_chapter_index(&state.chapter_text);
    if state.chapter_index > last {
        state.chapter_index = last;
    }

    let _ = app.emit("change-chapter-text", text);
    let _ = app.emit("change-chapter-index", state.chapter_index);
    Ok(())
}

#[tauri::command]
fn get_chapter_index(state: tauri::State<'_, Mutex<AppState>>) -> usize {
    state.lock().map(|state| state.chapter_index).unwrap_or_default()
}

#[tauri::command]
fn set_chapter_index(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<AppState>>,
    index: usize,
) -> Result<ChapterIndexResult, String> {
    let mut state = state.lock().map_err(|error| error.to_string())?;
    let result = set_chapter_index_inner(&mut state, index);
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
    let current = state.chapter_index as isize;
    let next = current.saturating_add(num).max(0) as usize;
    let result = set_chapter_index_inner(&mut state, next);
    let _ = app.emit("change-chapter-index", result.index);
    Ok(result)
}

fn set_chapter_index_inner(state: &mut AppState, index: usize) -> ChapterIndexResult {
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
fn spawn_global_mouse_events(app: tauri::AppHandle, event_seen: Arc<AtomicBool>) -> Result<(), String> {
    let is_button_down = Arc::new(AtomicBool::new(false));
    let cursor_position = Arc::new(Mutex::new((0i32, 0i32)));
    let normalized_position = Arc::new(Mutex::new((0i32, 0i32)));
    let app_for_events = app.clone();

    let is_button_down_for_events = Arc::clone(&is_button_down);
    let cursor_position_for_events = Arc::clone(&cursor_position);
    let normalized_position_for_events = Arc::clone(&normalized_position);
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
fn spawn_global_mouse_events(_app: tauri::AppHandle, _event_seen: Arc<AtomicBool>) -> Result<(), String> {
    Ok(())
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
}

#[cfg(not(target_os = "macos"))]
fn apply_macos_overlay_window_level(_window: &tauri::WebviewWindow) {}

pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            get_settings,
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

                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_always_on_top(true);
                    let _ = window.set_ignore_cursor_events(true);
                    let _ = window.set_shadow(false);
                    let _ = window.set_decorations(false);
                    apply_overlay_window_bounds(&window);
                    apply_macos_overlay_window_level(&window);
                }

                let listener_app = app.handle().clone();
                let event_seen = Arc::new(AtomicBool::new(false));
                let event_seen_for_listener = Arc::clone(&event_seen);

                if std::env::var("OVERLAY_DISABLE_MOUSE_LISTENER").ok().as_deref() != Some("1") {
                    let _watchdog_app = app.handle().clone();
                    thread::spawn(move || {
                        if let Err(error) = spawn_global_mouse_events(listener_app, event_seen_for_listener) {
                            let message = format!(
                                "Failed to start global mouse monitoring: {error}. Please allow Accessibility for this app in System Settings > Privacy & Security > Accessibility."
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
