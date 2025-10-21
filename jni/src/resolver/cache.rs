use std::{
    cell::RefCell,
    ffi::{CStr, CString},
    sync::Once,
    thread::LocalKey,
};

use typed_jni_core::{AttachHook, FieldID, JNIEnv, JavaVM, LocalRef, MethodID, StrongRef, WeakGlobalRef, sys};

pub type Slot<const N: usize, T> = RefCell<Option<Box<uluru::LRUCache<T, N>>>>;

const fn new_slot<const N: usize, T>() -> Slot<N, T> {
    RefCell::new(None)
}

fn clear_slot<const N: usize, T>(slot: &'static LocalKey<Slot<N, T>>) {
    let _ = slot.try_with(|v| v.take());
}

struct MethodWithClass {
    vm: *mut sys::JavaVM,
    class: CString,
    name: CString,
    sig: CString,
    class_obj: WeakGlobalRef<'static>,
    method: sys::jmethodID,
}

thread_local! {
    static CACHED_STATIC_METHODS_WITH_CLASS: Slot<8, MethodWithClass> = const { new_slot() };
    static CACHED_INSTANCE_METHODS_WITH_CLASS: Slot<8, MethodWithClass> = const { new_slot() };
}

pub fn find_class_and_method<'env, const STATIC: bool>(
    env: &'env JNIEnv,
    cls: &CStr,
    name: &CStr,
    sig: &CStr,
) -> Option<(LocalRef<'env>, MethodID<STATIC>)> {
    let slot = if STATIC {
        &CACHED_STATIC_METHODS_WITH_CLASS
    } else {
        &CACHED_INSTANCE_METHODS_WITH_CLASS
    };

    slot.with_borrow_mut(|v| unsafe {
        let cache = v.as_mut()?;

        let vm = env.vm();
        let entry = cache.find(|v| v.vm == vm.as_raw_ptr() && v.class == cls && v.name == name && v.sig == sig)?;

        let cls = env.new_local_ref(&entry.class_obj)?;
        let id = MethodID::from_raw(entry.method);

        Some((cls, id))
    })
}

pub fn put_class_and_method<const STATIC: bool, R: StrongRef>(
    env: &JNIEnv,
    cls: &CStr,
    name: &CStr,
    sig: &CStr,
    cls_obj: &R,
    method: MethodID<STATIC>,
) {
    setup_cache();

    let slot = if STATIC {
        &CACHED_STATIC_METHODS_WITH_CLASS
    } else {
        &CACHED_INSTANCE_METHODS_WITH_CLASS
    };

    slot.with_borrow_mut(|v| unsafe {
        let cache = v.get_or_insert_default();

        let vm = env.vm();
        let entry = MethodWithClass {
            vm: vm.as_raw_ptr(),
            class: cls.to_owned(),
            name: name.to_owned(),
            sig: sig.to_owned(),
            class_obj: core::mem::transmute::<WeakGlobalRef<'_>, WeakGlobalRef<'static>>(env.new_weak_global_ref(cls_obj)?),
            method: method.as_raw_ptr(),
        };

        cache.insert(entry);

        Some(())
    });
}

struct Member<T> {
    vm: *mut sys::JavaVM,
    class: WeakGlobalRef<'static>,
    name: CString,
    sig: CString,
    member: T,
}

fn find_member<R: StrongRef, T: Copy>(
    slot: &'static LocalKey<Slot<32, Member<T>>>,
    env: &JNIEnv,
    cls: &R,
    name: &CStr,
    sig: &CStr,
) -> Option<T> {
    slot.with_borrow_mut(|v| {
        let cache = v.as_mut()?;

        let vm = env.vm();
        let entry = cache.find(|v| {
            v.vm == vm.as_raw_ptr() && v.name == name && v.sig == sig && env.is_same_object(Some(&v.class), Some(cls))
        })?;

        Some(entry.member)
    })
}

fn put_member<R: StrongRef, T>(
    slot: &'static LocalKey<Slot<32, Member<T>>>,
    env: &JNIEnv,
    cls: &R,
    name: &CStr,
    sig: &CStr,
    member: T,
) {
    setup_cache();

    slot.with_borrow_mut(|v| unsafe {
        let cache = v.get_or_insert_default();

        let vm = env.vm();
        let entry = Member {
            vm: vm.as_raw_ptr(),
            class: core::mem::transmute::<WeakGlobalRef<'_>, WeakGlobalRef<'static>>(env.new_weak_global_ref(cls)?),
            name: name.to_owned(),
            sig: sig.to_owned(),
            member,
        };

        cache.insert(entry);

        Some(())
    });
}

thread_local! {
    static CACHED_STATIC_METHODS: Slot<32, Member<sys::jmethodID>> = const { new_slot() };
    static CACHED_INSTANCE_METHODS: Slot<32, Member<sys::jmethodID>> = const { new_slot() };
}

pub fn find_method<const STATIC: bool, R: StrongRef>(env: &JNIEnv, cls: &R, name: &CStr, sig: &CStr) -> Option<MethodID<STATIC>> {
    let method = if STATIC {
        find_member(&CACHED_STATIC_METHODS, env, cls, name, sig)?
    } else {
        find_member(&CACHED_INSTANCE_METHODS, env, cls, name, sig)?
    };

    unsafe { Some(MethodID::from_raw(method)) }
}

pub fn put_method<const STATIC: bool, R: StrongRef>(env: &JNIEnv, cls: &R, name: &CStr, sig: &CStr, method: MethodID<STATIC>) {
    if STATIC {
        put_member(&CACHED_STATIC_METHODS, env, cls, name, sig, method.as_raw_ptr())
    } else {
        put_member(&CACHED_INSTANCE_METHODS, env, cls, name, sig, method.as_raw_ptr())
    }
}

thread_local! {
    static CACHED_STATIC_FIELDS: Slot<32, Member<sys::jfieldID>> = const { new_slot() };
    static CACHED_INSTANCE_FIELDS: Slot<32, Member<sys::jfieldID>> = const { new_slot() };
}

pub fn find_field<const STATIC: bool, R: StrongRef>(env: &JNIEnv, cls: &R, name: &CStr, sig: &CStr) -> Option<FieldID<STATIC>> {
    let field = if STATIC {
        find_member(&CACHED_STATIC_FIELDS, env, cls, name, sig)?
    } else {
        find_member(&CACHED_INSTANCE_FIELDS, env, cls, name, sig)?
    };

    unsafe { Some(FieldID::from_raw(field)) }
}

pub fn put_field<const STATIC: bool, R: StrongRef>(env: &JNIEnv, cls: &R, name: &CStr, sig: &CStr, field: FieldID<STATIC>) {
    if STATIC {
        put_member(&CACHED_STATIC_FIELDS, env, cls, name, sig, field.as_raw_ptr())
    } else {
        put_member(&CACHED_INSTANCE_FIELDS, env, cls, name, sig, field.as_raw_ptr())
    }
}

fn setup_cache() {
    static mut PREV_HOOK: Option<AttachHook> = None;

    fn cleanup_cache_with_vm(vm: &JavaVM) {
        unsafe {
            clear_slot(&CACHED_STATIC_METHODS_WITH_CLASS);
            clear_slot(&CACHED_INSTANCE_METHODS_WITH_CLASS);
            clear_slot(&CACHED_STATIC_METHODS);
            clear_slot(&CACHED_INSTANCE_METHODS);
            clear_slot(&CACHED_STATIC_FIELDS);
            clear_slot(&CACHED_INSTANCE_FIELDS);

            if let Some(hook) = PREV_HOOK {
                hook(vm)
            }
        }
    }

    static ONCE: Once = Once::new();
    ONCE.call_once(|| unsafe {
        let prev = JavaVM::set_detach_hook(cleanup_cache_with_vm);

        PREV_HOOK = prev;
    })
}
