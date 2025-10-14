use alloc::{
    ffi::CString,
    format,
    string::{String, ToString},
};
use core::fmt::{Display, Formatter};

use crate::{Args, Context, Field, FromRaw, JavaThrowable, LocalObject, Method, Signature, StrongRef, Type};

#[cfg(feature = "cache")]
mod cache {
    use std::cell::RefCell;

    use uluru::LRUCache;

    use crate::{Context, JavaThrowable, LocalObject, StrongRef, Weak};

    const MAX_CACHED_PER_THREAD: usize = 32;

    struct Entry {
        class: Weak,
        types_id: usize,
        name: &'static str,
        member: *const (),
    }

    thread_local! {
        static CACHED: RefCell<LRUCache<Entry, MAX_CACHED_PER_THREAD>> = RefCell::new(LRUCache::new());
    }

    pub fn find_member<
        'ctx,
        C: StrongRef,
        M: Copy,
        F: FnOnce(Option<*const ()>) -> Result<(M, *const ()), LocalObject<'ctx, JavaThrowable>>,
    >(
        ctx: &'ctx Context,
        class: &C,
        name: &'static str,
        find: F,
    ) -> Result<M, LocalObject<'ctx, JavaThrowable>> {
        CACHED.with(|entries| {
            let mut entries = entries.borrow_mut();

            let types_id = find_member::<C, M, F> as *const () as usize;

            let cached = entries.find(|e| {
                e.types_id == types_id
                    && name.as_ptr() == e.name.as_ptr()
                    && name.len() == e.name.len()
                    && ctx.is_same_object(Some(&e.class), Some(class))
            });
            match cached {
                Some(e) => Ok(find(Some(e.member))?.0),
                None => {
                    let (member, cache) = find(None)?;

                    entries.insert(Entry {
                        class: ctx.new_weak_global_ref(class).unwrap(),
                        types_id,
                        name,
                        member: cache,
                    });

                    Ok(member)
                }
            }
        })
    }
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

pub fn find_method<'a, 'ctx, const STATIC: bool, C: StrongRef, A: Args<'a>, R: Type>(
    ctx: &'ctx Context,
    class: &C,
    name: &'static str,
) -> Result<Method<STATIC>, LocalObject<'ctx, JavaThrowable>>
where
    A::Array<Signature>: AsRef<[Signature]>,
{
    let do_find = || -> Result<Method<STATIC>, LocalObject<'ctx, JavaThrowable>> {
        ctx.find_method(
            class,
            CString::new(name).unwrap(),
            CString::new(method_signature_of(A::signatures().as_ref(), &R::SIGNATURE)).unwrap(),
        )
        .map_err(|err| unsafe { LocalObject::from_raw(err) })
    };

    #[cfg(feature = "cache")]
    return {
        use crate::IntoRaw;

        cache::find_member(ctx, class, name, |cached| match cached {
            Some(ptr) => unsafe { Ok((Method::from_raw(ptr as _), ptr)) },
            None => {
                let m = do_find()?;

                Ok((m, m.into_raw() as *const ()))
            }
        })
    };

    #[cfg(not(feature = "cache"))]
    do_find()
}

pub fn find_field<'a, 'ctx, const STATIC: bool, C: StrongRef, T: Type>(
    ctx: &'ctx Context,
    class: &C,
    name: &'static str,
) -> Result<Field<STATIC>, LocalObject<'ctx, JavaThrowable>> {
    let do_find = || -> Result<Field<STATIC>, LocalObject<'ctx, JavaThrowable>> {
        ctx.find_field(
            class,
            CString::new(name).unwrap(),
            CString::new(T::SIGNATURE.to_string()).unwrap(),
        )
        .map_err(|err| unsafe { LocalObject::from_raw(err) })
    };

    #[cfg(feature = "cache")]
    return {
        use crate::IntoRaw;

        cache::find_member(ctx, class, name, |cached| match cached {
            Some(ptr) => unsafe { Ok((Field::from_raw(ptr as _), ptr)) },
            None => {
                let f = do_find()?;

                Ok((f, f.into_raw() as _))
            }
        })
    };

    #[cfg(not(feature = "cache"))]
    do_find()
}

#[cfg(all(feature = "cache", test))]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicPtr, Ordering},
    };

    fn test_atomic_ordering(set_order: Ordering, fetch_order: Ordering) {
        const THREADS: usize = 100;
        const COUNT_PER_THREADS: usize = 1_000_000;

        struct Holder {
            count: usize,
        }

        let atomic: Arc<AtomicPtr<Holder>> = Arc::new(AtomicPtr::new(Box::into_raw(Box::new(Holder { count: 0 }))));

        let handles = (0..THREADS).map(|_| {
            let atomic = atomic.clone();

            std::thread::spawn(move || {
                for _ in 0..COUNT_PER_THREADS {
                    let ptr = loop {
                        match atomic.fetch_update(set_order, fetch_order, |ptr| {
                            if ptr.is_null() { None } else { Some(std::ptr::null_mut()) }
                        }) {
                            Ok(ptr) => break ptr,
                            Err(_) => std::hint::spin_loop(),
                        }
                    };

                    unsafe {
                        (*ptr).count += 1;
                    }

                    atomic.store(ptr, set_order);
                }
            })
        });

        for handle in handles {
            handle.join().unwrap();
        }

        unsafe {
            assert_eq!((*atomic.load(fetch_order)).count, THREADS * COUNT_PER_THREADS);
        }
    }

    #[test]
    fn test_atomic_ordering_relaxed() {
        test_atomic_ordering(Ordering::Relaxed, Ordering::Relaxed);
    }

    #[test]
    fn test_atomic_ordering_acqrel() {
        test_atomic_ordering(Ordering::Release, Ordering::Acquire);
    }
}
