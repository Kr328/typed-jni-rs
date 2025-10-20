mod object;
mod primitive;
mod primitive_impls;

use core::marker::PhantomData;

use typed_jni_core::{JNIEnv, StrongRef};

pub use self::{object::*, primitive::*};
use crate::{LocalObject, Object, ObjectType, Signature, Type, TypedRef, builtin::JavaThrowable};

/// A type descriptor of an array.
pub struct Array<T: Type + 'static>(pub PhantomData<T>);

impl<T: Type + 'static> Type for Array<T> {
    const SIGNATURE: Signature = Signature::Array(&T::SIGNATURE);
}

impl<T: Type + 'static> ObjectType for Array<T> {}

/// Extension methods for typed arrays.
pub trait TypedArrayExt {
    /// Get the length of an array.
    fn typed_get_array_length<R: StrongRef, T: ObjectType>(
        &self,
        array: &Object<R, Array<T>>,
    ) -> Result<i32, LocalObject<'_, JavaThrowable>>;
}

impl<'vm> TypedArrayExt for JNIEnv<'vm> {
    fn typed_get_array_length<R: StrongRef, T: Type + 'static>(
        &self,
        array: &Object<R, Array<T>>,
    ) -> Result<i32, LocalObject<'_, JavaThrowable>> {
        unsafe { self.get_array_length(&**array).map_err(|err| LocalObject::from_ref(err)) }
    }
}
