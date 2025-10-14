use core::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::sys::{JNI_OK, JNI_VERSION_1_4, JNIEnv, JavaVM};

static VM: AtomicPtr<JavaVM> = AtomicPtr::new(null_mut());

pub fn attach_vm(vm: *mut JavaVM) {
    VM.store(vm, Ordering::Relaxed);
}

pub fn require_vm() -> *mut JavaVM {
    let vm = VM.load(Ordering::Relaxed);
    if vm.is_null() {
        panic!("JavaVM not attached");
    }
    vm
}

pub unsafe fn current<'a>() -> Option<*mut JNIEnv> {
    let vm = require_vm();
    let mut env: *mut JNIEnv = null_mut();

    unsafe {
        let ret = (**vm).GetEnv?(vm, (&mut env as *mut *mut JNIEnv).cast(), JNI_VERSION_1_4 as i32);
        if ret == JNI_OK { Some(env) } else { None }
    }
}

pub struct AttachGuard {
    env: *mut JNIEnv,
    need_detach: bool,
}

impl AttachGuard {
    pub fn env(&self) -> *mut JNIEnv {
        self.env
    }
}

impl Drop for AttachGuard {
    fn drop(&mut self) {
        unsafe {
            if self.need_detach {
                let vm = require_vm();

                (**vm).DetachCurrentThread.unwrap()(vm);
            }
        }
    }
}

pub unsafe fn attach() -> AttachGuard {
    unsafe {
        match current() {
            None => {
                let vm = require_vm();
                let mut env: *mut JNIEnv = null_mut();

                let attached = (**vm).AttachCurrentThread.unwrap()(vm, (&mut env as *mut *mut JNIEnv).cast(), null_mut());
                if attached == JNI_OK {
                    AttachGuard { env, need_detach: true }
                } else {
                    panic!("BROKEN: unable to attach current thread.")
                }
            }
            Some(ctx) => AttachGuard {
                env: ctx,
                need_detach: false,
            },
        }
    }
}
