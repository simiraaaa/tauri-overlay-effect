#![allow(improper_ctypes_definitions)]
use crate::macos::common::*;
use crate::rdev::{Event, GrabError};
use core_graphics::event::{CGEventTapLocation, CGEventType};
use std::os::raw::c_void;
use std::ptr;

static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut(Event) -> Option<Event>>> = None;

#[link(name = "Cocoa", kind = "framework")]
extern "C" {}

unsafe extern "C" fn raw_callback(
    _proxy: CGEventTapProxy,
    _type: CGEventType,
    cg_event: CGEventRef,
    _user_info: *mut c_void,
) -> CGEventRef {
    // println!("Event ref {:?}", cg_event_ptr);
    // let cg_event: CGEvent = transmute_copy::<*mut c_void, CGEvent>(&cg_event_ptr);
    let opt = KEYBOARD_STATE.lock();
    if let Ok(mut keyboard) = opt {
        let _ = with_cg_event(cg_event, |cg_event| {
            if let Some(event) = convert(_type, cg_event, &mut keyboard) {
                // SAFETY: GLOBAL_CALLBACK is written only from this initialization thread
                // before the run loop starts, and we read it only from this callback thread.
                let mut callback = unsafe { (&raw mut GLOBAL_CALLBACK).as_mut() };
                if let Some(callback) = callback.as_mut().and_then(|cb| cb.as_mut()) {
                    if callback(event).is_none() {
                        cg_event.set_type(CGEventType::Null);
                    }
                }
            }
        });
    }
    cg_event
}

pub fn grab<T>(callback: T) -> Result<(), GrabError>
where
    T: FnMut(Event) -> Option<Event> + 'static,
{
    unsafe {
        GLOBAL_CALLBACK = Some(Box::new(callback));
        let tap = CGEventTapCreate(
            CGEventTapLocation::HID, // HID, Session, AnnotatedSession,
            kCGHeadInsertEventTap,
            CGEventTapOption::Default,
            kCGEventMaskForAllEvents,
            raw_callback,
            ptr::null_mut(),
        );
        if tap.is_null() {
            return Err(GrabError::EventTapError);
        }
        let _loop = CFMachPortCreateRunLoopSource(ptr::null(), tap, 0);
        if _loop.is_null() {
            return Err(GrabError::LoopSourceError);
        }

        let current_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(current_loop, _loop, kCFRunLoopCommonModes);

        CGEventTapEnable(tap, true);
        CFRunLoopRun();
    }
    Ok(())
}
