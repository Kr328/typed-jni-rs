mod get;
mod set;

use typed_jni_core::{JNIEnv, StrongRef};

pub use self::{get::Got, set::Value};
use crate::{LocalObject, Type, TypedRef, builtin::JavaThrowable, resolver};

/// Extension methods for typed field access.
pub trait TypedFieldAccessExt {
    /// Gets the value of a field of the given object.
    fn typed_get_field<'a, 'env, R, T>(&'env self, this: &'a T, name: &'a str) -> Result<R, LocalObject<'env, JavaThrowable>>
    where
        R: Got<'env> + Type,
        T: TypedRef,
        T::Target: StrongRef + Sized;

    /// Sets the value of a field of the given object.
    fn typed_set_field<'env, V, T>(&'env self, this: &T, name: &str, value: V) -> Result<(), LocalObject<'env, JavaThrowable>>
    where
        V: Value + Type,
        T: TypedRef,
        T::Target: StrongRef + Sized;
}

impl<'vm> TypedFieldAccessExt for JNIEnv<'vm> {
    fn typed_get_field<'a, 'env, R, T>(&'env self, this: &'a T, name: &'a str) -> Result<R, LocalObject<'env, JavaThrowable>>
    where
        R: Got<'env> + Type,
        T: TypedRef,
        T::Target: StrongRef + Sized,
    {
        unsafe {
            let s = R::SIGNATURE;

            if T::STATIC {
                let field = resolver::helper::resolve_field_by_this::<true, _>(self, &**this, name, s)?;

                R::get_of(self, &**this, field)
            } else {
                let field = resolver::helper::resolve_field_by_this::<false, _>(self, &**this, name, s)?;

                R::get_of(self, &**this, field)
            }
        }
    }

    fn typed_set_field<'env, V, T>(&'env self, this: &T, name: &str, value: V) -> Result<(), LocalObject<'env, JavaThrowable>>
    where
        V: Value + Type,
        T: TypedRef,
        T::Target: StrongRef + Sized,
    {
        unsafe {
            let s = V::SIGNATURE;

            if T::STATIC {
                let field = resolver::helper::resolve_field_by_this::<true, _>(self, &**this, name, s)?;

                value.set_on(self, &**this, field)
            } else {
                let field = resolver::helper::resolve_field_by_this::<false, _>(self, &**this, name, s)?;

                value.set_on(self, &**this, field)
            }
        }
    }
}
