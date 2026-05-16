#[cfg(target_os = "macos")]
mod platform {
  use std::sync::OnceLock;
  use objc::declare::ClassDecl;
  use objc::runtime::{Class, Object, Sel};
  use objc::{class, msg_send, sel, sel_impl};
  use cocoa::foundation::NSString;

  static SUSPEND_CB: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();
  static RESUME_CB: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();

  extern "C" fn on_sleep(_this: &Object, _sel: Sel, _note: *mut Object) {
    if let Some(cb) = SUSPEND_CB.get() {
      cb();
    }
  }

  extern "C" fn on_wake(_this: &Object, _sel: Sel, _note: *mut Object) {
    if let Some(cb) = RESUME_CB.get() {
      cb();
    }
  }

  extern "C" fn on_lock(_this: &Object, _sel: Sel, _note: *mut Object) {
    if let Some(cb) = SUSPEND_CB.get() {
      cb();
    }
  }

  extern "C" fn on_unlock(_this: &Object, _sel: Sel, _note: *mut Object) {
    if let Some(cb) = RESUME_CB.get() {
      cb();
    }
  }

  pub fn register(suspend: Box<dyn Fn() + Send + Sync>, resume: Box<dyn Fn() + Send + Sync>) {
    let _ = SUSPEND_CB.set(suspend);
    let _ = RESUME_CB.set(resume);

    let superclass = class!(NSObject);
    let mut decl = ClassDecl::new("MoirunPowerObserver", superclass).unwrap();

    unsafe {
      decl.add_method(
        sel!(onSleep:),
        on_sleep as extern "C" fn(&Object, Sel, *mut Object),
      );
      decl.add_method(
        sel!(onWake:),
        on_wake as extern "C" fn(&Object, Sel, *mut Object),
      );
      decl.add_method(
        sel!(onLock:),
        on_lock as extern "C" fn(&Object, Sel, *mut Object),
      );
      decl.add_method(
        sel!(onUnlock:),
        on_unlock as extern "C" fn(&Object, Sel, *mut Object),
      );
    }

    let cls = decl.register();
    let observer: *mut Object = unsafe { msg_send![cls, new] };

    let center: *mut Object = unsafe { msg_send![class!(NSNotificationCenter), defaultCenter] };
    let workspace: *mut Object = unsafe { msg_send![class!(NSWorkspace), sharedWorkspace] };

    let sleep_name: *mut Object =
      unsafe { NSString::alloc(std::ptr::null_mut()).init_str("NSWorkspaceWillSleepNotification") };
    let wake_name: *mut Object =
      unsafe { NSString::alloc(std::ptr::null_mut()).init_str("NSWorkspaceDidWakeNotification") };

    unsafe {
      let _: () = msg_send![
        center,
        addObserver:observer
        selector:sel!(onSleep:)
        name:sleep_name
        object:workspace
      ];
      let _: () = msg_send![
        center,
        addObserver:observer
        selector:sel!(onWake:)
        name:wake_name
        object:workspace
      ];
    }

    let dcenter: *mut Object = unsafe {
      let cls = Class::get("NSDistributedNotificationCenter").unwrap();
      msg_send![cls, defaultCenter]
    };

    let lock_name: *mut Object =
      unsafe { NSString::alloc(std::ptr::null_mut()).init_str("com.apple.screenIsLocked") };
    let unlock_name: *mut Object =
      unsafe { NSString::alloc(std::ptr::null_mut()).init_str("com.apple.screenIsUnlocked") };

    unsafe {
      let _: () = msg_send![
        dcenter,
        addObserver:observer
        selector:sel!(onLock:)
        name:lock_name
        object:0 as *mut Object
      ];
      let _: () = msg_send![
        dcenter,
        addObserver:observer
        selector:sel!(onUnlock:)
        name:unlock_name
        object:0 as *mut Object
      ];
    }
  }
}

#[cfg(not(target_os = "macos"))]
mod platform {
  pub fn register(
    _suspend: Box<dyn Fn() + Send + Sync>,
    _resume: Box<dyn Fn() + Send + Sync>,
  ) {
    // 非 macOS 平台暂为空实现
  }
}

pub fn init(
  suspend: impl Fn() + Send + Sync + 'static,
  resume: impl Fn() + Send + Sync + 'static,
) {
  platform::register(Box::new(suspend), Box::new(resume));
}
