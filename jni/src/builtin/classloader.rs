use crate::{ObjectType, Signature, Type};

pub struct JavaClassLoader;

impl Type for JavaClassLoader {
    const SIGNATURE: Signature = Signature::Object("java/lang/ClassLoader");
}

impl ObjectType for JavaClassLoader {}
