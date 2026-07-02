use crate::macos::common::*;
use crate::rdev::{Event, ListenError};
use cocoa::base::nil;
use cocoa::foundation::NSAutoreleasePool;
use core_graphics::event::{CGEventTapLocation, CGEventType};
use std::panic::{self, AssertUnwindSafe};
use std::os::raw::c_void;

static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut(Event)>> = None;

#[link(name = "Cocoa", kind = "framework")]
extern "C" {}

unsafe extern "C" fn raw_callback(
    _proxy: CGEventTapProxy,
    _type: CGEventType,
    cg_event: CGEventRef,
    _user_info: *mut c_void,
) -> CGEventRef {
    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        // println!("Event ref {:?}", cg_event_ptr);
        // let cg_event: CGEvent = transmute_copy::<*mut c_void, CGEvent>(&cg_event_ptr);
        let opt = KEYBOARD_STATE.lock();
        if let Ok(mut keyboard) = opt {
            if let Some(event) = convert(_type, &cg_event, &mut keyboard) {
                // SAFETY: GLOBAL_CALLBACK is written only from this initialization thread
                // before the run loop starts, and we read it only from this callback thread.
                let mut callback = unsafe { (&raw mut GLOBAL_CALLBACK).as_mut() };
                if let Some(callback) = callback.as_mut().and_then(|cb| cb.as_mut()) {
                    callback(event);
                }
            }
        }
    }));
    // println!("Event ref END {:?}", cg_event_ptr);
    // cg_event_ptr
    cg_event
}

pub fn listen<T>(callback: T) -> Result<(), ListenError>
where
    T: FnMut(Event) + 'static,
{
    unsafe {
        GLOBAL_CALLBACK = Some(Box::new(callback));

        let _pool = NSAutoreleasePool::new(nil);
        let tap = CGEventTapCreate(
            CGEventTapLocation::HID, // HID, Session, AnnotatedSession,
            kCGHeadInsertEventTap,
            CGEventTapOption::ListenOnly,
            kCGEventMaskForAllEvents,
            raw_callback,
            nil,
        );
        if tap.is_null() {
            return Err(ListenError::EventTapError);
        }
        let _loop = CFMachPortCreateRunLoopSource(nil, tap, 0);
        if _loop.is_null() {
            return Err(ListenError::LoopSourceError);
        }

        let current_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(current_loop, _loop, kCFRunLoopCommonModes);

        CGEventTapEnable(tap, true);
        CFRunLoopRun();
    }
    Ok(())
}
