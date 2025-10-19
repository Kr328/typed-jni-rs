use core::fmt::{Debug, Display, Formatter};

use typed_jni_core::{GlobalRef, LocalRef, Ref};

use crate::{Object, ObjectType, Signature, Type, TypedObjectExt};

pub struct JavaThrowable;

impl Type for JavaThrowable {
    const SIGNATURE: Signature = Signature::Object("java/lang/Throwable");
}

impl ObjectType for JavaThrowable {}

impl<'env> Display for Object<LocalRef<'env>, JavaThrowable> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let s = self.env().typed_to_string(self);

        match s {
            Ok(s) => f.write_str(&s),
            Err(_) => f.write_str("<exception>"),
        }
    }
}

impl<'vm> Display for Object<GlobalRef<'vm>, JavaThrowable> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let s = self
            .vm()
            .with_attached_thread(true, |env| env.typed_to_string(self).ok())
            .ok()
            .flatten();

        match s {
            Some(s) => f.write_str(&s),
            None => f.write_str("<exception>"),
        }
    }
}

impl<R: Ref> Debug for Object<R, JavaThrowable>
where
    Self: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self, f)
    }
}

#[cfg(feature = "std")]
impl<R: Ref> std::error::Error for Object<R, JavaThrowable> where Self: Display {}
