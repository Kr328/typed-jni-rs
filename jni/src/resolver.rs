use alloc::{
    boxed::Box,
    ffi::CString,
    format,
    string::{String, ToString},
};
use core::{
    fmt::{Display, Formatter},
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{Args, Context, Field, Local, Method, Signature, StrongRef, JavaThrowable, Type, Weak};

const MAX_MEMBER_CACHE_PER_SLOT: usize = 128;

struct Entry {
    class: Weak,
    types_id: usize,
    name: &'static str,
    member: *const (),
}

struct Slot {
    entries: uluru::LRUCache<Entry, MAX_MEMBER_CACHE_PER_SLOT>,
    next: *mut Slot,
}

static SLOTS: AtomicPtr<Slot> = AtomicPtr::new(null_mut());

fn get_or_alloc_slot() -> &'static mut Slot {
    unsafe {
        loop {
            match SLOTS.load(Ordering::Relaxed).as_mut() {
                None => {
                    break Box::leak(Box::new(Slot {
                        entries: uluru::LRUCache::new(),
                        next: null_mut(),
                    }));
                }
                Some(current) => match SLOTS.compare_exchange(current, current.next, Ordering::Relaxed, Ordering::Relaxed) {
                    Ok(_) => {
                        current.next = null_mut();

                        break current;
                    }
                    Err(_) => continue,
                },
            }
        }
    }
}

fn put_slot(slot: &'static mut Slot) {
    loop {
        let next = SLOTS.load(Ordering::Relaxed);

        slot.next = next;

        match SLOTS.compare_exchange(next, slot, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(_) => continue,
        }
    }
}

fn use_a_slot<R, F>(f: F) -> R
where
    for<'a> F: FnOnce(&'a mut &'static mut Slot) -> R,
{
    let mut slot = get_or_alloc_slot();

    let r = f(&mut slot);

    put_slot(slot);

    r
}

pub fn method_signature_of(args: &[Signature], ret: &Signature) -> String {
    struct ArgsSignature<'a>(&'a [Signature]);

    impl<'a> Display for ArgsSignature<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> alloc::fmt::Result {
            for x in self.0.iter() {
                x.fmt(f)?
            }

            Ok(())
        }
    }

    format!("({}){}", ArgsSignature(args), ret)
}

pub fn find_method<'a, 'ctx, const STATIC: bool, const ARGS: usize, C: StrongRef, A: Args<'a, ARGS>, R: Type>(
    ctx: &'ctx Context,
    class: &C,
    name: &'static str,
) -> Result<Method<STATIC>, JavaThrowable<false, Local<'ctx>>> {
    use_a_slot(|slot| {
        let types_id = find_method::<STATIC, ARGS, C, A, R> as *const () as usize;

        let cached = slot.entries.find(|e| {
            e.types_id == types_id && name.as_ptr() == e.name.as_ptr() && ctx.is_same_object(Some(&e.class), Some(class))
        });
        match cached {
            Some(e) => unsafe { Ok(Method::<STATIC>::from_raw(e.member as _)) },
            None => {
                let method = ctx.find_method(
                    class,
                    CString::new(name).unwrap(),
                    CString::new(method_signature_of(&A::signatures(), &R::SIGNATURE)).unwrap(),
                )?;

                slot.entries.insert(Entry {
                    class: class.downgrade_weak(),
                    types_id,
                    name,
                    member: method.as_raw() as _,
                });

                Ok(method)
            }
        }
    })
}

pub fn find_field<'a, 'ctx, const STATIC: bool, C: StrongRef, T: Type>(
    ctx: &'ctx Context,
    class: &C,
    name: &'static str,
) -> Result<Field<STATIC>, JavaThrowable<false, Local<'ctx>>> {
    use_a_slot(|slot| {
        let types_id = find_field::<STATIC, C, T> as *const () as usize;

        let cached = slot.entries.find(|e| {
            e.types_id == types_id && name.as_ptr() == e.name.as_ptr() && ctx.is_same_object(Some(&e.class), Some(class))
        });
        match cached {
            Some(e) => unsafe { Ok(Field::<STATIC>::from_raw(e.member as _)) },
            None => {
                let field = ctx.find_field(
                    class,
                    CString::new(name).unwrap(),
                    CString::new(T::SIGNATURE.to_string()).unwrap(),
                )?;

                slot.entries.insert(Entry {
                    class: class.downgrade_weak(),
                    types_id,
                    name,
                    member: field.as_raw() as _,
                });

                Ok(field)
            }
        }
    })
}
