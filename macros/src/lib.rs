#![no_std]

#[macro_export]
macro_rules! define_java_class {
    ($name:ident, $class:literal) => {
        pub struct $name;

        impl ::typed_jni::Type for $name {
            const SIGNATURE: ::typed_jni::Signature = ::typed_jni::Signature::Object($class);
        }

        impl ::typed_jni::ObjectType for $name {}
    };
}
