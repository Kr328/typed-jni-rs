use core::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::sys::JavaVM;

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
