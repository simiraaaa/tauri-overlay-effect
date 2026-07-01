use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{Emitter, Manager};

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
struct MouseEvent<'a> {
    position: &'a str,
    #[serde(rename = "type")]
    event_type: &'a str,
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

fn spawn_dummy_mouse_events(app: tauri::AppHandle) {
    thread::spawn(move || {
        let mut x = 160;
        let mut direction = 1;

        loop {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit(
                    "global-mouse",
                    MouseEvent {
                        position: "left",
                        event_type: "down",
                        x,
                        y: 160,
                    },
                );

                thread::sleep(Duration::from_millis(90));

                let _ = window.emit(
                    "global-mouse",
                    MouseEvent {
                        position: "left",
                        event_type: "up",
                        x,
                        y: 160,
                    },
                );
            }

            x += 40 * direction;
            if x > 520 || x < 160 {
                direction *= -1;
            }

            thread::sleep(Duration::from_millis(1200));
        }
    });
}

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
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_ignore_cursor_events(true);
                let _ = window.set_shadow(false);
            }

            spawn_dummy_mouse_events(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run tauri application");
}
