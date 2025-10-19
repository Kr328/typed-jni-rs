use alloc::{borrow::Cow, ffi::CString, format};

use typed_jni_core::{Arg, JNIEnv, StrongRef};

use crate::{
    LocalClass, LocalObject, Object, ObjectType, Signature, TypedRef,
    builtin::{JavaClass, JavaClassLoader, JavaThrowable},
    resolver,
};

fn class_name_of(signature: Signature, internal_name: bool) -> Cow<'static, str> {
    match signature {
        Signature::Void => Cow::Borrowed("V"),
        Signature::Boolean => Cow::Borrowed("Z"),
        Signature::Byte => Cow::Borrowed("B"),
        Signature::Char => Cow::Borrowed("C"),
        Signature::Short => Cow::Borrowed("S"),
        Signature::Int => Cow::Borrowed("I"),
        Signature::Long => Cow::Borrowed("J"),
        Signature::Float => Cow::Borrowed("F"),
        Signature::Double => Cow::Borrowed("D"),
        Signature::Object(cls) => {
            if internal_name {
                Cow::Borrowed(cls)
            } else {
                Cow::Owned(cls.replace('/', "."))
            }
        }
        Signature::Array(s) => {
            let mut sig = format!("[{}", s);

            if !internal_name {
                sig = sig.replace('/', ".");
            }

            Cow::Owned(sig)
        }
    }
}

/// Extension methods for typed class maintenance.
pub trait TypedClassExt {
    /// Finds a class with the given type `T`.
    fn typed_find_class<T: ObjectType>(&self) -> Result<LocalClass<'_, T>, LocalObject<'_, JavaThrowable>>;

    /// Finds a class with the given type `T` in the given class loader.
    fn typed_find_class_in_class_loader<T: ObjectType, R: StrongRef>(
        &self,
        class_loader: &Object<R, JavaClassLoader>,
    ) -> Result<LocalClass<'_, T>, LocalObject<'_, JavaThrowable>>;

    /// Returns the class loader of the given class object.
    fn typed_get_class_loader<R: StrongRef>(
        &self,
        cls: &Object<R, JavaClass>,
    ) -> Result<Option<LocalObject<'_, JavaClassLoader>>, LocalObject<'_, JavaThrowable>>;
}

impl<'vm> TypedClassExt for JNIEnv<'vm> {
    fn typed_find_class<T: ObjectType>(&self) -> Result<LocalClass<'_, T>, LocalObject<'_, JavaThrowable>> {
        let class_name = CString::new(class_name_of(T::SIGNATURE, true).into_owned()).unwrap();

        unsafe {
            self.find_class(&*class_name)
                .map(|v| LocalClass::from_ref(v))
                .map_err(|err| LocalObject::from_ref(err))
        }
    }

    fn typed_find_class_in_class_loader<T: ObjectType, R: StrongRef>(
        &self,
        class_loader: &Object<R, JavaClassLoader>,
    ) -> Result<LocalClass<'_, T>, LocalObject<'_, JavaThrowable>> {
        unsafe {
            let (_, method) = resolver::resolve_class_and_method_raw::<true>(
                self,
                c"java/lang/Class",
                c"forName",
                c"(Ljava/lang/String;ZLjava/lang/ClassLoader;)Ljava/lang/Class;",
            )?;

            let class_name = self.new_string(class_name_of(T::SIGNATURE, false));

            self.call_object_method(
                &**class_loader,
                method,
                [
                    Arg::Object(Some(&class_name)),
                    Arg::Boolean(true),
                    Arg::Object(Some(&**class_loader)),
                ],
            )
            .map(|v| LocalClass::from_ref(v.expect("BROKEN: Class.forName returning null")))
            .map_err(|err| LocalObject::from_ref(err))
        }
    }

    fn typed_get_class_loader<R: StrongRef>(
        &self,
        cls: &Object<R, JavaClass>,
    ) -> Result<Option<LocalObject<'_, JavaClassLoader>>, LocalObject<'_, JavaThrowable>> {
        unsafe {
            let (_, method) = resolver::resolve_class_and_method_raw::<false>(
                self,
                c"java/lang/Class",
                c"getClassLoader",
                c"()Ljava/lang/ClassLoader;",
            )?;

            self.call_object_method(&**cls, method, [])
                .map(|v| v.map(|v| LocalObject::from_ref(v)))
                .map_err(|err| LocalObject::from_ref(err))
        }
    }
}
