use alloc::collections::btree_map::Entry;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    ffi::{CStr, CString},
    sync::Once,
};

use typed_jni_core::{AttachHook, FieldID, JNIEnv, JavaVM, LocalRef, MethodID, StrongRef, WeakGlobalRef, sys};
use uluru::LRUCache;

const METHOD_WITH_CLASS_CAPACITY: usize = 8;
const MEMBER_ONLY_CAPACITY: usize = 32;

struct MethodWithClass {
    cls: WeakGlobalRef<'static>,
    method: sys::jmethodID,
    sig: CString,
    class: CString,
    name: CString,
}

struct Member<T> {
    cls: WeakGlobalRef<'static>,
    member: T,
    name: CString,
    sig: CString,
}

#[derive(Default)]
struct Cached {
    static_method_with_class: LRUCache<MethodWithClass, METHOD_WITH_CLASS_CAPACITY>,
    instance_method_with_class: LRUCache<MethodWithClass, METHOD_WITH_CLASS_CAPACITY>,
    static_methods: LRUCache<Member<sys::jmethodID>, MEMBER_ONLY_CAPACITY>,
    instance_methods: LRUCache<Member<sys::jmethodID>, MEMBER_ONLY_CAPACITY>,
    static_fields: LRUCache<Member<sys::jfieldID>, MEMBER_ONLY_CAPACITY>,
    instance_fields: LRUCache<Member<sys::jfieldID>, MEMBER_ONLY_CAPACITY>,
}

#[derive(Default)]
struct AttachOnClean(Option<Box<Cached>>);

impl Drop for AttachOnClean {
    fn drop(&mut self) {
        if let Some(c) = self.0.take() {
            let mut attached: BTreeMap<*mut sys::JavaVM, bool> = BTreeMap::new();

            let mut handle_vm = |vm: &'static JavaVM| unsafe {
                if let Entry::Vacant(v) = attached.entry(vm.as_raw_ptr()) {
                    if vm.current_env().is_some() {
                        v.insert(false);
                    } else {
                        vm.attach_current_thread(false).unwrap();

                        v.insert(true);
                    }
                }
            };

            for c in [&c.static_method_with_class, &c.instance_method_with_class] {
                for entry in c.iter() {
                    handle_vm(entry.cls.vm());
                }
            }
            for c in [&c.static_methods, &c.instance_methods] {
                for entry in c.iter() {
                    handle_vm(entry.cls.vm());
                }
            }
            for c in [&c.static_fields, &c.instance_fields] {
                for entry in c.iter() {
                    handle_vm(entry.cls.vm());
                }
            }

            drop(c);

            for (vm, attached) in attached {
                if attached {
                    unsafe {
                        let _ = JavaVM::from_raw(vm).detach_current_thread();
                    }
                }
            }
        }
    }
}

thread_local! {
    static CACHED: RefCell<AttachOnClean> = const { RefCell::new(AttachOnClean(None)) };
}

fn setup_cache() {
    static mut PREV_HOOK: Option<AttachHook> = None;

    fn cleanup_cache_with_vm(vm: &JavaVM) {
        unsafe {
            let cached = CACHED.try_with(|v| v.take());
            drop(cached);

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

pub fn find_class_and_method<'env, const STATIC: bool>(
    env: &'env JNIEnv,
    cls: &CStr,
    name: &CStr,
    sig: &CStr,
) -> Option<(LocalRef<'env>, MethodID<STATIC>)> {
    CACHED.with_borrow_mut(|AttachOnClean(v)| {
        let v = v.as_mut()?;

        let cache = if STATIC {
            &mut v.static_method_with_class
        } else {
            &mut v.instance_method_with_class
        };

        let vm = env.vm();
        let entry =
            cache.find(|v| v.cls.vm().as_raw_ptr() == vm.as_raw_ptr() && v.class == cls && v.name == name && v.sig == sig)?;

        let cls = env.new_local_ref(&entry.cls)?;
        let id = unsafe { MethodID::from_raw(entry.method) };

        Some((cls, id))
    })
}

pub fn put_class_and_method<const STATIC: bool, R: StrongRef>(
    env: &JNIEnv,
    class: &CStr,
    name: &CStr,
    sig: &CStr,
    cls: &R,
    method: MethodID<STATIC>,
) {
    setup_cache();

    CACHED.with_borrow_mut(|AttachOnClean(v)| {
        let v = v.get_or_insert_default();

        let cache = if STATIC {
            &mut v.static_method_with_class
        } else {
            &mut v.instance_method_with_class
        };

        let cls = match env.new_weak_global_ref(cls) {
            Some(cls) => cls,
            None => return,
        };

        let entry = MethodWithClass {
            cls: unsafe { core::mem::transmute::<WeakGlobalRef<'_>, WeakGlobalRef<'static>>(cls) },
            method: method.as_raw_ptr(),
            class: class.to_owned(),
            name: name.to_owned(),
            sig: sig.to_owned(),
        };

        cache.insert(entry);
    });
}

fn find_member<R: StrongRef, T: Copy>(
    cache: &mut LRUCache<Member<T>, MEMBER_ONLY_CAPACITY>,
    env: &JNIEnv,
    cls: &R,
    name: &CStr,
    sig: &CStr,
) -> Option<T> {
    let vm = env.vm();
    let entry = cache.find(|v| {
        v.cls.vm().as_raw_ptr() == vm.as_raw_ptr()
            && v.name == name
            && v.sig == sig
            && env.is_same_object(Some(&v.cls), Some(cls))
    })?;

    Some(entry.member)
}

fn put_member<R: StrongRef, T>(
    cache: &mut LRUCache<Member<T>, MEMBER_ONLY_CAPACITY>,
    env: &JNIEnv,
    cls: &R,
    name: &CStr,
    sig: &CStr,
    member: T,
) {
    let cls = match env.new_weak_global_ref(cls) {
        Some(cls) => cls,
        None => return,
    };

    let entry = Member {
        cls: unsafe { core::mem::transmute::<WeakGlobalRef<'_>, WeakGlobalRef<'static>>(cls) },
        name: name.to_owned(),
        sig: sig.to_owned(),
        member,
    };

    cache.insert(entry);
}

pub fn find_method<const STATIC: bool, R: StrongRef>(env: &JNIEnv, cls: &R, name: &CStr, sig: &CStr) -> Option<MethodID<STATIC>> {
    CACHED.with_borrow_mut(|AttachOnClean(v)| {
        let cache = v.as_mut()?;

        let method = if STATIC {
            find_member(&mut cache.static_methods, env, cls, name, sig)?
        } else {
            find_member(&mut cache.instance_methods, env, cls, name, sig)?
        };

        unsafe { Some(MethodID::from_raw(method)) }
    })
}

pub fn put_method<const STATIC: bool, R: StrongRef>(env: &JNIEnv, cls: &R, name: &CStr, sig: &CStr, method: MethodID<STATIC>) {
    setup_cache();

    CACHED.with_borrow_mut(|AttachOnClean(v)| {
        let cache = v.get_or_insert_default();

        if STATIC {
            put_member(&mut cache.static_methods, env, cls, name, sig, method.as_raw_ptr())
        } else {
            put_member(&mut cache.instance_methods, env, cls, name, sig, method.as_raw_ptr())
        }
    });
}

pub fn find_field<const STATIC: bool, R: StrongRef>(env: &JNIEnv, cls: &R, name: &CStr, sig: &CStr) -> Option<FieldID<STATIC>> {
    CACHED.with_borrow_mut(|AttachOnClean(v)| {
        let cache = v.as_mut()?;

        let method = if STATIC {
            find_member(&mut cache.static_fields, env, cls, name, sig)?
        } else {
            find_member(&mut cache.instance_fields, env, cls, name, sig)?
        };

        unsafe { Some(FieldID::from_raw(method)) }
    })
}

pub fn put_field<const STATIC: bool, R: StrongRef>(env: &JNIEnv, cls: &R, name: &CStr, sig: &CStr, field: FieldID<STATIC>) {
    setup_cache();

    CACHED.with_borrow_mut(|AttachOnClean(v)| {
        let cache = v.get_or_insert_default();

        if STATIC {
            put_member(&mut cache.static_fields, env, cls, name, sig, field.as_raw_ptr())
        } else {
            put_member(&mut cache.instance_fields, env, cls, name, sig, field.as_raw_ptr())
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::resolver::cache::Cached;

    #[test]
    fn print_cached_size() {
        println!("{}", size_of::<Cached>())
    }
}
