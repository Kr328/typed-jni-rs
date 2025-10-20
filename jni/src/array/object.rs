use typed_jni_core::{JNIEnv, StrongRef};

use crate::{Array, Class, LocalObject, Object, ObjectType, TypedArrayExt, TypedRef, builtin::JavaThrowable};

/// Extension methods for typed object arrays.
pub trait TypedObjectArrayExt: TypedArrayExt {
    /// Create a new object array.
    fn typed_new_array<T: ObjectType, R: StrongRef>(
        &self,
        cls: &Class<R, T>,
        len: i32,
    ) -> Result<LocalObject<'_, Array<T>>, LocalObject<'_, JavaThrowable>>;

    /// Create a new object array with initial value.
    fn typed_new_array_with_initial<T: ObjectType, R: StrongRef, IR: StrongRef>(
        &self,
        cls: &Class<R, T>,
        len: i32,
        initial: &Object<IR, T>,
    ) -> Result<LocalObject<'_, Array<T>>, LocalObject<'_, JavaThrowable>>;

    /// Get an element from object array.
    fn typed_get_array_element<T: ObjectType, R: StrongRef>(
        &self,
        arr: &Object<R, Array<T>>,
        index: i32,
    ) -> Result<Option<LocalObject<'_, T>>, LocalObject<'_, JavaThrowable>>;

    /// Set an element to object array.
    fn typed_set_array_element<T: ObjectType, R: StrongRef>(
        &self,
        arr: &Object<R, Array<T>>,
        index: i32,
        value: Option<&Object<R, T>>,
    ) -> Result<(), LocalObject<'_, JavaThrowable>>;
}

impl<'vm> TypedObjectArrayExt for JNIEnv<'vm> {
    fn typed_new_array<T: ObjectType, R: StrongRef>(
        &self,
        cls: &Class<R, T>,
        len: i32,
    ) -> Result<LocalObject<'_, Array<T>>, LocalObject<'_, JavaThrowable>> {
        unsafe {
            self.new_object_array(&**cls, len)
                .map(|v| LocalObject::from_ref(v))
                .map_err(|err| LocalObject::from_ref(err))
        }
    }

    fn typed_new_array_with_initial<T: ObjectType, R: StrongRef, IR: StrongRef>(
        &self,
        cls: &Class<R, T>,
        len: i32,
        initial: &Object<IR, T>,
    ) -> Result<LocalObject<'_, Array<T>>, LocalObject<'_, JavaThrowable>> {
        unsafe {
            self.new_object_array_with_initial(&**cls, len, &**initial)
                .map(|v| LocalObject::from_ref(v))
                .map_err(|err| LocalObject::from_ref(err))
        }
    }

    fn typed_get_array_element<T: ObjectType, R: StrongRef>(
        &self,
        arr: &Object<R, Array<T>>,
        index: i32,
    ) -> Result<Option<LocalObject<'_, T>>, LocalObject<'_, JavaThrowable>> {
        unsafe {
            self.get_object_array_element(&**arr, index)
                .map(|v| v.map(|v| LocalObject::from_ref(v)))
                .map_err(|err| LocalObject::from_ref(err))
        }
    }

    fn typed_set_array_element<T: ObjectType, R: StrongRef>(
        &self,
        arr: &Object<R, Array<T>>,
        index: i32,
        value: Option<&Object<R, T>>,
    ) -> Result<(), LocalObject<'_, JavaThrowable>> {
        unsafe {
            self.set_object_array_element(&**arr, index, value.map(|v| &**v))
                .map_err(|err| LocalObject::from_ref(err))
        }
    }
}
