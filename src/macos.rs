//! Finder sends open-document Apple events, not command-line arguments.
//! Keep eframe/winit's application delegate intact and handle only those events.
use std::cell::RefCell;
use std::path::Path;

use objc2::rc::Retained;
use objc2::{define_class, msg_send, sel, MainThreadOnly};
use objc2_app_kit::{NSApplication, NSApplicationWillFinishLaunchingNotification};
use objc2_foundation::{
    MainThreadMarker, NSAppleEventDescriptor, NSAppleEventManager, NSNotification,
    NSNotificationCenter, NSObject, NSObjectProtocol,
};

const CORE_EVENT_CLASS: u32 = u32::from_be_bytes(*b"aevt");
const OPEN_DOCUMENTS: u32 = u32::from_be_bytes(*b"odoc");
const DIRECT_OBJECT: u32 = u32::from_be_bytes(*b"----");

#[derive(Default)]
struct PendingDocument {
    path: Option<String>,
    context: Option<egui::Context>,
}

thread_local! {
    static PENDING: RefCell<PendingDocument> = RefCell::new(PendingDocument::default());
}

fn queue_documents(documents: &NSAppleEventDescriptor) {
    // This viewer displays one document at a time, like its drag-and-drop path.
    // Use the first supported file when Finder sends a multiple-file selection.
    for index in 1..=documents.numberOfItems() {
        let Some(path) = documents
            .descriptorAtIndex(index)
            .and_then(|item| item.fileURLValue())
            .filter(|url| url.isFileURL())
            .and_then(|url| url.path())
            .map(|path| path.to_string())
        else {
            continue;
        };
        if !Path::new(&path)
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| {
                ext.eq_ignore_ascii_case("parquet") || ext.eq_ignore_ascii_case("parq")
            })
        {
            continue;
        }
        PENDING.with_borrow_mut(|pending| {
            pending.path = Some(path);
            if let Some(ctx) = &pending.context {
                ctx.request_repaint();
            }
        });
        break;
    }
}

pub fn set_context(ctx: &egui::Context) {
    PENDING.with_borrow_mut(|pending| {
        pending.context = Some(ctx.clone());
        if pending.path.is_some() {
            ctx.request_repaint();
        }
    });
}

pub fn take_pending_file() -> Option<String> {
    PENDING.with_borrow_mut(|pending| pending.path.take())
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements. All Apple event callbacks
    // and access to the pending document occur on the application's main thread.
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    struct OpenDocumentHandler;

    unsafe impl NSObjectProtocol for OpenDocumentHandler {}

    impl OpenDocumentHandler {
        // SAFETY: Notification-center selectors take exactly one NSNotification.
        #[unsafe(method(applicationWillFinishLaunching:))]
        fn will_finish_launching(&self, _notification: &NSNotification) {
            self.register();
        }

        // SAFETY: This is NSAppleEventManager's documented two-descriptor handler
        // signature. We don't retain or modify the incoming event or reply.
        #[unsafe(method(handleOpenDocuments:withReplyEvent:))]
        fn handle_open_documents(&self, event: &NSAppleEventDescriptor, _reply: &NSAppleEventDescriptor) {
            // SAFETY: paramDescriptorForKeyword: accepts an AEKeyword (UInt32)
            // and returns a nullable NSAppleEventDescriptor.
            let documents: Option<Retained<NSAppleEventDescriptor>> = unsafe {
                msg_send![event, paramDescriptorForKeyword: DIRECT_OBJECT]
            };
            if let Some(documents) = documents {
                queue_documents(&documents);
            }
        }
    }
);

impl OpenDocumentHandler {
    fn register(&self) {
        let manager = NSAppleEventManager::sharedAppleEventManager();
        // SAFETY: The selector has NSAppleEventManager's two-descriptor handler
        // signature; the event class and ID are Apple's UInt32 four-char codes.
        unsafe {
            let _: () = msg_send![&*manager,
                setEventHandler: self,
                andSelector: sel!(handleOpenDocuments:withReplyEvent:),
                forEventClass: CORE_EVENT_CLASS,
                andEventID: OPEN_DOCUMENTS
            ];
        }
    }
}

pub struct DocumentHandler {
    // Retain the receiver until the event loop exits.
    _handler: Retained<OpenDocumentHandler>,
    manager: Retained<NSAppleEventManager>,
    notifications: Retained<NSNotificationCenter>,
}

impl DocumentHandler {
    pub fn install() -> Self {
        let mtm = MainThreadMarker::new().expect("macOS UI must run on the main thread");
        // Initialize AppKit before overriding its default open-document handler.
        let _app = NSApplication::sharedApplication(mtm);
        // SAFETY: NSObject init returns an initialized instance of our subclass.
        let handler: Retained<OpenDocumentHandler> =
            unsafe { msg_send![OpenDocumentHandler::alloc(mtm), init] };
        let manager = NSAppleEventManager::sharedAppleEventManager();
        // AppKit replaces the open-document handler in finishLaunching. Observe
        // will-finish-launching, after that setup and before the first document
        // event; registering here in main is too early.
        let notifications = NSNotificationCenter::defaultCenter();
        // SAFETY: The retained main-thread receiver implements the one-argument
        // notification selector. Drop removes the observer before releasing it.
        unsafe {
            notifications.addObserver_selector_name_object(
                &handler,
                sel!(applicationWillFinishLaunching:),
                Some(NSApplicationWillFinishLaunchingNotification),
                Some(&_app),
            );
        }
        Self {
            _handler: handler,
            manager,
            notifications,
        }
    }
}

impl Drop for DocumentHandler {
    fn drop(&mut self) {
        // SAFETY: Remove only our retained notification receiver; both event
        // arguments match the registered UInt32 event codes.
        unsafe {
            self.notifications.removeObserver(&self._handler);
            let _: () = msg_send![&*self.manager,
                removeEventHandlerForEventClass: CORE_EVENT_CLASS,
                andEventID: OPEN_DOCUMENTS
            ];
        }
        PENDING.with_borrow_mut(|pending| *pending = PendingDocument::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use objc2_foundation::{NSString, NSURL};

    fn documents(paths: &[&str]) -> Retained<NSAppleEventDescriptor> {
        let list = NSAppleEventDescriptor::listDescriptor();
        for path in paths {
            let url = NSURL::fileURLWithPath(&NSString::from_str(path));
            list.insertDescriptor_atIndex(&NSAppleEventDescriptor::descriptorWithFileURL(&url), 0);
        }
        list
    }

    #[test]
    fn queues_launch_documents_and_preserves_special_characters() {
        let path = "/tmp/行情 space # 100% = XRPUSDT.parquet";
        queue_documents(&documents(&[path]));
        set_context(&egui::Context::default());
        assert_eq!(take_pending_file().as_deref(), Some(path));
        assert_eq!(take_pending_file(), None);
    }

    #[test]
    fn handles_later_requests_and_first_supported_selection() {
        set_context(&egui::Context::default());
        queue_documents(&documents(&[
            "/tmp/skip.txt",
            "/tmp/first.PARQ",
            "/tmp/second.parquet",
        ]));
        assert_eq!(take_pending_file().as_deref(), Some("/tmp/first.PARQ"));
        queue_documents(&documents(&["/tmp/later.parquet"]));
        assert_eq!(take_pending_file().as_deref(), Some("/tmp/later.parquet"));
        queue_documents(&documents(&["/tmp/unsupported.csv"]));
        assert_eq!(take_pending_file(), None);
    }
}
