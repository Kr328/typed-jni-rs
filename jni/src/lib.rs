#![cfg_attr(not(feature = "std"), no_std)]

//! # JNI Bindings for Rust with Typed References
//!
//! This crate provides Rust bindings for the Java Native Interface (JNI) with typed references.
//!
//! ## Features
//!
//! * `std` - Enables the use standard library. (default)
//! * `cache` - Enables the use cache for class and member lookups. (default, requires `std`)
//!
//! ## Getting Started
//!
//! Assume you have Java classes `org.example.Example1` and `org.example.Example2` with following declarations:
//!
//! ```java
//! package org.example;
//!
//! public class Example1 {
//!     private final String str;
//!
//!     public Example1(String str) {
//!         this.str = str;
//!     }
//!
//!     public void hello() {
//!         System.out.println(str);
//!
//!         nativeHello(str);
//!     }
//!
//!     private native void nativeHello(String str);
//!     private native String getStr();
//! }
//!
//! public class Example2 {
//!     private static final Example1 staticExample1 = new Example1("Hello World!!!");
//!
//!     private final Example1 example1;
//!
//!     public Example2(Example1 example1) {
//!         this.example1 = example1;
//!     }
//!
//!     public static void staticHello(int n) {
//!         for (i = 0; i < n; i++) {
//!             staticExample1.hello();
//!         }
//!     }
//!
//!     public void hello(int n) {
//!         for (i = 0; i < n; i++) {
//!             example1.hello();
//!         }
//!     }
//! }
//! ```
//!
//! ### Define Java Classes in Rust
//!
//! Firstly, add necessary classes defines in your Rust code.
//!
//! ```rust
//! define_java_class!(JavaExample1, "org.example.Example1"); // `Java` prefix of name is not required
//! define_java_class!(JavaExample2, "org.example.Example2");
//! ```
//!
//! ### Access Java in Rust
//!
//! Now you can access Java classes in type safe way.
//!
//! ```rust
//! let env: &JNIEnv = { /* attach jvm or get env from java native function */ };
//!
//! // create Example object
//! let example1_cls: LocalClass<JavaExample1> = env.typed_find_class().unwrap();
//! let example1_obj: LocalObject<JavaExample1> = env
//!     .typed_new_object(&example1_cls, (env.typed_new_string("Hello World!"),))
//!     .unwrap();
//!
//! // call hello method
//! env.typed_call_method::<(), _, _>(&example1_obj, "hello", ()).unwrap();
//!
//! // get str field
//! let str: LocalObject<JavaString> = env.typed_get_field(&example1_obj, "str").unwrap();
//! let str: String = env.typed_get_string(&str);
//!
//! // create Example2 object
//! let example2_cls: LocalClass<JavaExample2> = env.typed_find_class().unwrap();
//! let example2_obj: LocalObject<JavaExample2> = env.typed_new_object(&example2_cls, (example1_obj,)).unwrap();
//!
//! // call staticHello method
//! env.typed_call_method::<(), _, _>(&example2_cls, "staticHello", (3i32,)).unwrap();
//!
//! // call hello method
//! env.typed_call_method::<(), _, _>(&example2_obj, "hello", (3i32,)).unwrap();
//!
//! // get staticExample1 field
//! let static_example1: LocalObject<JavaExample1> = env.typed_get_field(&example2_cls, "staticExample1").unwrap();
//!
//! // get example1 field
//! let example1: LocalObject<JavaExample1> = env.typed_get_field(&example2_obj, "example1").unwrap();
//! ```
//!
//! ### Access Rust in Java
//!
//! You should define native methods in Rust code with following signature.
//!
//! ```rust
//! use typed_jni::TrampolineObject;
//!
//! #[unsafe(no_mangle)]
//! pub extern "system" fn Java_org_example_Example1_nativeHello<'env>(env: &'env JNIEnv, obj: TrampolineObject<'env, JavaExample1>, str: TrampolineObject<'env, JavaString>) {
//!     let str: String = env.typed_get_string(&str);
//!
//!     println!("{}", str);
//! }
//!
//! #[unsafe(no_mangle)]
//! pub extern "system" fn Java_org_example_Example1_getStr<'env>(env: &'env JNIEnv, obj: TrampolineObject<'env, JavaExample1>) -> TrampolineObject<'env, JavaString> {
//!     env.typed_new_string("native string").into_trampoline()
//! }
//! ```
//!
//! Then load it in Java code.
//!
//! ```java
//! System.loadLibrary("example1");
//! ```
//!
//! **NOTE**: All object reference in native function **parameters** and **return value** should be `TrampolineObject` or `TrampolineClass`, it is ffi safe.
//!

extern crate alloc;

mod array;
pub mod builtin;
mod call;
mod class;
mod field;
mod object;
mod reference;
mod resolver;
mod string;
mod throwable;

use ::core::{
    fmt::{Display, Formatter},
    marker::PhantomData,
    ops::Deref,
};
pub use typed_jni_core as core;
use typed_jni_core::{GlobalRef, LocalRef, Ref, TrampolineRef, WeakGlobalRef};

pub use self::{array::*, call::*, class::*, field::*, object::*, reference::*, string::*, throwable::*};

/// A signature of a JNI type.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Signature {
    Void,
    Boolean,
    Byte,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Object(&'static str),
    Array(&'static Signature),
}

impl Display for Signature {
    fn fmt(&self, f: &mut Formatter<'_>) -> alloc::fmt::Result {
        match self {
            Signature::Void => f.write_str("V"),
            Signature::Boolean => f.write_str("Z"),
            Signature::Byte => f.write_str("B"),
            Signature::Char => f.write_str("C"),
            Signature::Short => f.write_str("S"),
            Signature::Int => f.write_str("I"),
            Signature::Long => f.write_str("J"),
            Signature::Float => f.write_str("F"),
            Signature::Double => f.write_str("D"),
            Signature::Object(name) => f.write_fmt(format_args!("L{};", name)),
            Signature::Array(inner) => f.write_fmt(format_args!("[{}", inner)),
        }
    }
}

/// A Java type.
pub trait Type {
    const SIGNATURE: Signature;
}

impl<T: Type> Type for &T {
    const SIGNATURE: Signature = T::SIGNATURE;
}

impl<T: Type> Type for Option<T> {
    const SIGNATURE: Signature = T::SIGNATURE;
}

/// A Java primitive type.
pub trait PrimitiveType: Type + 'static {}

/// A Java object type.
pub trait ObjectType: Type + 'static {}

impl Type for () {
    const SIGNATURE: Signature = Signature::Void;
}

macro_rules! impl_primitive_type {
    ($typ:ty, $signature:expr) => {
        impl Type for $typ {
            const SIGNATURE: Signature = $signature;
        }

        impl PrimitiveType for $typ {}
    };
}

impl_primitive_type!(bool, Signature::Boolean);
impl_primitive_type!(i8, Signature::Byte);
impl_primitive_type!(u16, Signature::Char);
impl_primitive_type!(i16, Signature::Short);
impl_primitive_type!(i32, Signature::Int);
impl_primitive_type!(i64, Signature::Long);
impl_primitive_type!(f32, Signature::Float);
impl_primitive_type!(f64, Signature::Double);

/// A reference to an object or class with a specific type.
pub trait TypedRef: Deref
where
    Self::Target: Ref,
{
    /// Whether the reference is static.
    const STATIC: bool;

    /// The type of the object or class.
    type Type: ObjectType;

    /// Creates a new reference from a raw reference.
    ///
    /// # Safety
    ///
    /// The reference must match [Self::Type].
    unsafe fn from_ref(reference: Self::Target) -> Self;

    /// Converts the reference to a raw reference.
    fn into_ref(self) -> Self::Target;
}

/// A reference to an object with a specific type.
#[repr(transparent)]
pub struct Object<R: Ref, T: ObjectType> {
    reference: R,
    _typ: PhantomData<T>,
}

/// A reference to a class with a specific type.
#[repr(transparent)]
pub struct Class<R: Ref, T: ObjectType> {
    reference: R,
    _typ: PhantomData<T>,
}

macro_rules! impl_typed_common {
    ($typ:ident, $is_static:literal) => {
        impl<R: Ref, T: ObjectType> Type for $typ<R, T> {
            const SIGNATURE: Signature = T::SIGNATURE;
        }

        impl<R: Ref, T: ObjectType> Deref for $typ<R, T> {
            type Target = R;

            fn deref(&self) -> &Self::Target {
                &self.reference
            }
        }

        impl<R: Ref, T: ObjectType> TypedRef for $typ<R, T> {
            const STATIC: bool = $is_static;

            type Type = T;

            unsafe fn from_ref(reference: R) -> Self {
                Self {
                    reference,
                    _typ: PhantomData,
                }
            }

            fn into_ref(self) -> R {
                self.reference
            }
        }

        impl<'env, T: ObjectType> $typ<LocalRef<'env>, T> {
            /// Converts the reference to a trampoline reference.
            pub fn into_trampoline(self) -> TrampolineObject<'env, T> {
                unsafe { TrampolineObject::from_ref(self.into_ref().into_trampoline()) }
            }
        }
    };
}

impl_typed_common!(Object, false);
impl_typed_common!(Class, true);

/// A local reference to an object with a specific type.
pub type LocalObject<'env, T> = Object<LocalRef<'env>, T>;
/// A trampoline reference to an object with a specific type.
pub type TrampolineObject<'env, T> = Object<TrampolineRef<'env>, T>;
/// A global reference to an object with a specific type.
pub type GlobalObject<'vm, T> = Object<GlobalRef<'vm>, T>;
/// A weak global reference to an object with a specific type.
pub type WeakGlobalObject<'vm, T> = Object<WeakGlobalRef<'vm>, T>;

/// A local reference to a class with a specific type.
pub type LocalClass<'env, T> = Class<LocalRef<'env>, T>;
/// A trampoline reference to a class with a specific type.
pub type TrampolineClass<'env, T> = Class<TrampolineRef<'env>, T>;
/// A global reference to a class with a specific type.
pub type GlobalClass<'vm, T> = Class<GlobalRef<'vm>, T>;
/// A weak global reference to a class with a specific type.
pub type WeakGlobalClass<'vm, T> = Class<WeakGlobalRef<'vm>, T>;

#[doc(hidden)]
pub const unsafe fn __class_name_to_internal_name_bytes<const N: usize>(s: &'static str) -> [u8; N] {
    let data = s.as_bytes();
    let mut ret = [0u8; N];

    let mut index = 0;
    while index < N {
        if data[index] == b'.' {
            ret[index] = b'/';
        } else {
            ret[index] = data[index];
        }

        index += 1;
    }

    ret
}

#[doc(hidden)]
pub const unsafe fn __bytes_to_str(bytes: &'static [u8]) -> &'static str {
    unsafe { ::core::str::from_utf8_unchecked(bytes) }
}

/// Defines a Java class as [`Type`] with the given name.
#[macro_export]
macro_rules! define_java_class {
    ($name:ident, $class:literal) => {
        pub struct $name;

        impl $crate::Type for $name {
            const SIGNATURE: $crate::Signature = $crate::Signature::Object(unsafe {
                const REPLACED: [u8; ($class).len()] = unsafe { $crate::__class_name_to_internal_name_bytes($class) };

                $crate::__bytes_to_str(&REPLACED)
            });
        }

        impl $crate::ObjectType for $name {}
    };
}
