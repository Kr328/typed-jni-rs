use alloc::{format, string::String};

use typed_jni_core::{Arg, JNIEnv, StrongRef};

use crate::{
    Class, LocalObject, Object, ObjectType, TypedRef,
    builtin::{JavaClass, JavaThrowable},
    resolver,
};

/// Extension methods for typed object maintenance.
pub trait TypedObjectExt {
    /// Returns the class of the object.
    fn typed_get_object_class<O>(&self, obj: &O) -> LocalObject<'_, JavaClass>
    where
        O: TypedRef,
        O::Target: StrongRef + Sized;

    /// Returns whether the object is an instance of the class.
    fn typed_is_instance_of<'env, FT: ObjectType, TT: ObjectType, FR: StrongRef, TR: StrongRef>(
        &'env self,
        obj: &Object<FR, FT>,
        cls: &Class<TR, TT>,
    ) -> bool;

    /// Returns whether the class is assignable from the super class.
    fn typed_is_assignable_from<'env, FT: ObjectType, TT: ObjectType, FR: StrongRef, TR: StrongRef>(
        &'env self,
        cls: &Class<FR, FT>,
        super_cls: &Class<TR, TT>,
    ) -> bool;

    /// Casts the object to the class.
    fn typed_cast<'env, FT: ObjectType, TT: ObjectType, FR: StrongRef, TR: StrongRef>(
        &'env self,
        obj: &Object<FR, FT>,
        cls: &Class<TR, TT>,
    ) -> Result<LocalObject<'env, TT>, LocalObject<'env, JavaThrowable>>;

    /// Returns the string representation of the object.
    fn typed_to_string<O>(&self, obj: &O) -> Result<String, LocalObject<'_, JavaThrowable>>
    where
        O: TypedRef,
        O::Target: StrongRef + Sized;

    /// Returns the hash code of the object.
    fn typed_hash_code<O>(&self, obj: &O) -> Result<i32, LocalObject<'_, JavaThrowable>>
    where
        O: TypedRef,
        O::Target: StrongRef + Sized;
}

impl<'vm> TypedObjectExt for JNIEnv<'vm> {
    fn typed_get_object_class<O>(&self, obj: &O) -> LocalObject<'_, JavaClass>
    where
        O: TypedRef,
        O::Target: StrongRef + Sized,
    {
        unsafe { LocalObject::from_ref(self.get_object_class(&**obj)) }
    }

    fn typed_is_instance_of<'env, FT: ObjectType, TT: ObjectType, FR: StrongRef, TR: StrongRef>(
        &'env self,
        obj: &Object<FR, FT>,
        cls: &Class<TR, TT>,
    ) -> bool {
        unsafe { self.is_instance_of(&**obj, &**cls) }
    }

    fn typed_is_assignable_from<'env, FT: ObjectType, TT: ObjectType, FR: StrongRef, TR: StrongRef>(
        &'env self,
        cls: &Class<FR, FT>,
        super_cls: &Class<TR, TT>,
    ) -> bool {
        unsafe { self.is_assignable_from(&**cls, &**super_cls) }
    }

    fn typed_cast<'env, FT: ObjectType, TT: ObjectType, TR: StrongRef, CR: StrongRef>(
        &'env self,
        obj: &Object<TR, FT>,
        cls: &Class<CR, TT>,
    ) -> Result<LocalObject<'env, TT>, LocalObject<'env, JavaThrowable>> {
        unsafe {
            if self.is_instance_of(&**obj, &**cls) {
                Ok(LocalObject::from_ref(
                    self.new_local_ref(&**obj).expect("BROKEN: create new local reference failed"),
                ))
            } else {
                let (cls, method) = resolver::resolve_class_and_method_raw(
                    self,
                    c"java/lang/ClassCastException",
                    c"<init>",
                    c"(Ljava/lang/String;)V",
                )?;

                match self.new_object(
                    &cls,
                    method,
                    [Arg::Object(Some(&self.new_string(format!(
                        "Object<{}> cannot cast to Class<{}>",
                        FT::SIGNATURE,
                        TT::SIGNATURE
                    ))))],
                ) {
                    Ok(ex) => Err(LocalObject::from_ref(ex)),
                    Err(err) => Err(LocalObject::from_ref(err)),
                }
            }
        }
    }

    fn typed_to_string<O>(&self, obj: &O) -> Result<String, LocalObject<'_, JavaThrowable>>
    where
        O: TypedRef,
        O::Target: StrongRef + Sized,
    {
        unsafe {
            let (_, method) =
                resolver::resolve_class_and_method_raw::<false>(self, c"java/lang/Object", c"toString", c"()Ljava/lang/String;")?;

            let s = self
                .call_object_method(&**obj, method, [])
                .map_err(|err| LocalObject::from_ref(err))?;

            let s = s.unwrap(); // TODO: throw null pointer exception

            Ok(self.get_string(&s))
        }
    }

    fn typed_hash_code<O>(&self, obj: &O) -> Result<i32, LocalObject<'_, JavaThrowable>>
    where
        O: TypedRef,
        O::Target: StrongRef + Sized,
    {
        unsafe {
            let (_, method) = resolver::resolve_class_and_method_raw::<false>(self, c"java/lang/Object", c"hashCode", c"()I")?;

            let c = self
                .call_int_method(&**obj, method, [])
                .map_err(|err| LocalObject::from_ref(err))?;

            Ok(c)
        }
    }
}
