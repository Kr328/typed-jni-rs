use typed_jni_core::Ref;

use crate::{Class, Object, ObjectType, Signature, Type, TypedRef};

pub struct JavaClass;

impl Type for JavaClass {
    const SIGNATURE: Signature = Signature::Object("java/lang/Class");
}

impl ObjectType for JavaClass {}

impl<R: Ref, T: ObjectType> Class<R, T> {
    pub fn into_class_object(self) -> Object<R, JavaClass> {
        unsafe { Object::from_ref(self.into_ref()) }
    }
}
