#![no_std]

#[macro_export]
macro_rules! define_java_class {
    ($name:ident, $class:literal) => {
        #[repr(transparent)]
        pub struct $name<const STATIC: bool, R: ::typed_jni::Ref>(R);

        impl<const STATIC: bool, R: ::typed_jni::Ref> ::typed_jni::RefConvert for $name<STATIC, R> {
            type Ref = R;
            type Converted<RR: ::typed_jni::Ref> = $name<STATIC, RR>;

            fn as_raw_ref(&self) -> &Self::Ref {
                &self.0
            }

            unsafe fn from_raw_ref<RR: ::typed_jni::Ref>(r: RR) -> Self::Converted<RR> {
                $name(r)
            }
        }

        #[allow(dead_code)]
        impl<const STATIC: bool, R: ::typed_jni::Ref> ::typed_jni::Value for $name<STATIC, R> {
            type Raw = R;
        }

        #[allow(dead_code)]
        impl<const STATIC: bool, R: ::typed_jni::Ref> ::typed_jni::FromRaw for $name<STATIC, R> {
            unsafe fn from_raw(raw: Self::Raw) -> Self {
                Self(raw)
            }
        }

        #[allow(dead_code)]
        impl<const STATIC: bool, R: ::typed_jni::Ref> ::typed_jni::IntoRaw for $name<STATIC, R> {
            fn into_raw(self) -> Self::Raw {
                self.0
            }
        }

        #[allow(dead_code)]
        impl<'a, const STATIC: bool, R: ::typed_jni::Ref> ::typed_jni::Value for &'a $name<STATIC, R> {
            type Raw = &'a R;
        }

        #[allow(dead_code)]
        impl<'a, const STATIC: bool, R: ::typed_jni::Ref> ::typed_jni::IntoRaw for &'a $name<STATIC, R> {
            fn into_raw(self) -> Self::Raw {
                &self.0
            }
        }

        #[allow(dead_code)]
        impl<R: ::typed_jni::Ref> ::typed_jni::Type for $name<false, R> {
            const SIGNATURE: ::typed_jni::Signature = ::typed_jni::Signature::Object($class);
        }

        #[allow(dead_code)]
        impl<R: ::typed_jni::Ref> ::typed_jni::Type for $name<true, R> {
            const SIGNATURE: ::typed_jni::Signature = ::typed_jni::Signature::Object("java/lang/Class");
        }

        #[allow(dead_code)]
        impl<const STATIC: bool, R: ::typed_jni::StrongRef> ::typed_jni::This<STATIC> for $name<STATIC, R> {
            type Ref = R;

            fn as_ref(&self) -> &Self::Ref {
                &self.0
            }
        }

        #[allow(dead_code)]
        impl<R: ::typed_jni::StrongRef> ::typed_jni::Class for $name<true, R> {
            type Object<RR: ::typed_jni::Ref> = $name<false, RR>;
        }
    };
}
