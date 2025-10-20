use typed_jni_core::Ref;

use crate::{Class, Object, ObjectType, Signature, Type, TypedRef};

pub struct JavaObject;

impl Type for JavaObject {
    const SIGNATURE: Signature = Signature::Object("java/lang/Object");
}

impl ObjectType for JavaObject {}

impl<R: Ref, T: ObjectType> Object<R, T> {
    pub fn into_object(self) -> Object<R, JavaObject> {
        unsafe { Object::from_ref(self.into_ref()) }
    }
}

impl<R: Ref, T: ObjectType> Class<R, T> {
    pub fn into_object(self) -> Object<R, JavaObject> {
        unsafe { Object::from_ref(self.into_ref()) }
    }
}
