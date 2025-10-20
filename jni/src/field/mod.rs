mod get;
mod set;

use typed_jni_core::{JNIEnv, StrongRef};

pub use self::{get::Got, set::Value};
use crate::{LocalObject, Type, TypedRef, builtin::JavaThrowable, resolver, resolver::helper::MemberKind};

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
            let name = resolver::helper::build_member_name(self, name, MemberKind::Field)?;
            let signature = resolver::helper::build_field_signature(self, R::SIGNATURE)?;

            if T::STATIC {
                let field = resolver::resolve_field::<true, _>(self, &**this, &*name, &signature)?;

                R::get_of(self, &**this, field)
            } else {
                let cls = self.get_object_class(&**this);

                let field = resolver::resolve_field::<false, _>(self, &cls, &*name, &signature)?;

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
            let name = resolver::helper::build_member_name(self, name, MemberKind::Field)?;
            let signature = resolver::helper::build_field_signature(self, V::SIGNATURE)?;

            if T::STATIC {
                let field = resolver::resolve_field::<true, _>(self, &**this, &*name, &signature)?;

                value.set_on(self, &**this, field)
            } else {
                let cls = self.get_object_class(&**this);

                let field = resolver::resolve_field::<false, _>(self, &cls, &*name, &signature)?;

                value.set_on(self, &**this, field)
            }
        }
    }
}
