//! Run with: cargo test --test macos_open_documents
//! Requires a logged-in macOS desktop session.
#[cfg(target_os = "macos")]
#[allow(dead_code)]
#[path = "../src/macos.rs"]
mod macos;

#[cfg(target_os = "macos")]
fn main() {
    use objc2::encode::{Encoding, RefEncode};
    use objc2::rc::Retained;
    use objc2::{msg_send, ClassType};
    use objc2_app_kit::NSApplication;
    use objc2_foundation::{
        MainThreadMarker, NSAppleEventDescriptor, NSAppleEventManager, NSString, NSURL,
    };
    use std::ffi::c_void;

    #[repr(C)]
    struct AEDesc {
        descriptor_type: u32,
        data_handle: *mut *mut c_void,
    }
    // SAFETY: Matches the Carbon AEDesc layout and Objective-C type encoding.
    unsafe impl RefEncode for AEDesc {
        const ENCODING_REF: Encoding = Encoding::Pointer(&Encoding::Struct(
            "AEDesc",
            &[
                Encoding::UInt,
                Encoding::Pointer(&Encoding::Pointer(&Encoding::Struct(
                    "OpaqueAEDataStorageType",
                    &[],
                ))),
            ],
        ));
    }

    let _handler = macos::DocumentHandler::install();
    let app = NSApplication::sharedApplication(MainThreadMarker::new().unwrap());
    // AppKit installs its own Apple event handlers during this phase. Testing
    // the queue alone cannot detect our registration being overwritten here.
    app.finishLaunching();

    for path in [
        "/tmp/行情 space # 100% = XRPUSDT.parquet",
        "/tmp/second document.parq",
    ] {
        let list = NSAppleEventDescriptor::listDescriptor();
        let url = NSURL::fileURLWithPath(&NSString::from_str(path));
        list.insertDescriptor_atIndex(&NSAppleEventDescriptor::descriptorWithFileURL(&url), 0);
        let target = NSAppleEventDescriptor::currentProcessDescriptor();
        // SAFETY: These signatures mirror Foundation's Apple event API. The
        // descriptors own the raw AEDesc storage for the entire dispatch call.
        unsafe {
            let event: Retained<NSAppleEventDescriptor> = msg_send![NSAppleEventDescriptor::class(),
                appleEventWithEventClass: u32::from_be_bytes(*b"aevt"),
                eventID: u32::from_be_bytes(*b"odoc"),
                targetDescriptor: &*target,
                returnID: -1_i16,
                transactionID: 0_i32
            ];
            let _: () = msg_send![&*event, setParamDescriptor: &*list, forKeyword: u32::from_be_bytes(*b"----")];
            let reply = NSAppleEventDescriptor::nullDescriptor();
            let raw_event: *const AEDesc = msg_send![&*event, aeDesc];
            let raw_reply: *const AEDesc = msg_send![&*reply, aeDesc];
            let manager = NSAppleEventManager::sharedAppleEventManager();
            let result: i16 = msg_send![&*manager,
                dispatchRawAppleEvent: raw_event,
                withRawReply: raw_reply as *mut AEDesc,
                handlerRefCon: std::ptr::null_mut::<c_void>()
            ];
            assert_eq!(result, 0, "Apple event dispatch failed");
        }
        assert_eq!(macos::take_pending_file().as_deref(), Some(path));
        assert_eq!(macos::take_pending_file(), None);
        macos::set_context(&egui::Context::default());
    }
    println!("AppKit launch and subsequent open-document dispatch passed.");
}

#[cfg(not(target_os = "macos"))]
fn main() {}
