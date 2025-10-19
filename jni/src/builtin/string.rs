use crate::{ObjectType, Signature, Type};

pub struct JavaString;

impl Type for JavaString {
    const SIGNATURE: Signature = Signature::Object("java/lang/String");
}

impl ObjectType for JavaString {}
