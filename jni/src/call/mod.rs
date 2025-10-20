mod args;
mod target;

use typed_jni_core::{JNIEnv, StrongRef};

pub use self::{args::*, target::Target};
use crate::{
    Class, LocalObject, ObjectType, Signature, Type, TypedRef, builtin::JavaThrowable, resolver, resolver::helper::MemberKind,
};

/// Extension methods for typed method call.
pub trait TypedCallExt {
    /// Calls a method with typed arguments.
    ///
    /// # This
    ///
    /// * `&Object<impl StrongRef, Type>` - A typed reference to an object. call as instance method.
    /// * `&Class<impl StrongRef, Type>` - A typed reference to a class. call as static method.
    ///
    /// # Args
    ///
    /// * `()` - No arguments.
    /// * `(impl ToArg,)` - A single argument that implements [`ToArg`].
    /// * `(impl ToArg, ...)` - Multiple arguments that implement [`ToArg`]. (Max 32 args)
    /// * `&[&dyn DynArg]` - Any number of arguments that implement [`ToArg`]. e.g. `&[0i32 as &dyn ToArg, 2i64, false]`
    ///
    /// # Returns
    ///
    /// * `impl Target` - A type that implements [`Target`].
    fn typed_call_method<'env, R, T, A>(&'env self, this: &T, name: &str, args: A) -> Result<R, LocalObject<'env, JavaThrowable>>
    where
        R: Target<'env> + Type,
        T: TypedRef,
        T::Target: StrongRef + Sized,
        A: Args;

    /// Calls a constructor with typed arguments.
    ///
    /// # This
    ///
    /// * `&Class<impl StrongRef, Type>` - A typed reference to a class.
    ///
    /// # Args
    ///
    /// * `()` - No arguments.
    /// * `(impl ToArg,)` - A single argument that implements [`ToArg`].
    /// * `(impl ToArg, ...)` - Multiple arguments that implement [`ToArg`]. (Max 32 args)
    /// * `&[&dyn DynArg]` - Any number of arguments that implement [`ToArg`]. e.g. `&[0i32 as &dyn ToArg, 2i64, false]`
    ///
    /// # Returns
    ///
    /// * `LocalObject<Type>` - A typed reference to the newly created object of Type.
    fn typed_new_object<T, R, A>(&self, cls: &Class<R, T>, args: A) -> Result<LocalObject<'_, T>, LocalObject<'_, JavaThrowable>>
    where
        R: StrongRef,
        T: ObjectType,
        A: Args;
}

impl<'vm> TypedCallExt for JNIEnv<'vm> {
    fn typed_call_method<'env, R, T, A>(&'env self, this: &T, name: &str, args: A) -> Result<R, LocalObject<'env, JavaThrowable>>
    where
        R: Target<'env> + Type,
        T: TypedRef,
        T::Target: StrongRef + Sized,
        A: Args,
    {
        unsafe {
            let name = resolver::helper::build_member_name(self, name, MemberKind::Method)?;
            let signature = resolver::helper::build_method_signature(self, R::SIGNATURE, args.signature())?;

            if T::STATIC {
                let method = resolver::resolve_method::<true, _>(self, &**this, &*name, &signature)?;

                args.apply_on(self, &**this, method)
            } else {
                let cls = self.get_object_class(&**this);

                let method = resolver::resolve_method::<false, _>(self, &cls, &*name, &signature)?;

                args.apply_on(self, &**this, method)
            }
        }
    }

    fn typed_new_object<T, R, A>(&self, cls: &Class<R, T>, args: A) -> Result<LocalObject<'_, T>, LocalObject<'_, JavaThrowable>>
    where
        R: StrongRef,
        T: ObjectType,
        A: Args,
    {
        unsafe {
            let signature = resolver::helper::build_method_signature(self, Signature::Void, args.signature())?;

            let method = resolver::resolve_method::<false, _>(self, &**cls, c"<init>", &signature)?;

            let target::NewObject(ret): target::NewObject<T> = args.apply_on(self, &**cls, method)?;

            Ok(ret)
        }
    }
}
