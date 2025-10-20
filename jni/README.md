# JNI Bindings for Rust with Typed References

This crate provides Rust bindings for the Java Native Interface (JNI) with typed references.

## Getting Started

Assume you have Java classes `org.example.Example1` and `org.example.Example2` with following declarations:

 ```java
 package org.example;
 public class Example1 {
     private final String str;
     public Example1(String str) {
         this.str = str;
     }
     public void hello() {
         System.out.println(str);
         nativeHello(str);
     }
     private native void nativeHello(String str);
     private native String getStr();
 }
 public class Example2 {
     private static final Example1 staticExample1 = new Example1("Hello World!!!");
     private final Example1 example1;
     public Example2(Example1 example1) {
         this.example1 = example1;
     }
     public static void staticHello(int n) {
         for (i = 0; i < n; i++) {
             staticExample1.hello();
         }
     }
     public void hello(int n) {
         for (i = 0; i < n; i++) {
             example1.hello();
         }
     }
 }
 ```

### Define Java Classes in Rust

Firstly, add necessary classes defines in your Rust code.

 ```rust
  use typed_jni::define_java_class;
define_java_class!(JavaExample1, "org.example.Example1"); // `Java` prefix of name is not required
define_java_class!(JavaExample2, "org.example.Example2");
 ```

### Access Java in Rust

Now you can access Java classes in type-safe way.

 ```rust
  use std::string::String;
use typed_jni::{
    LocalClass, LocalObject, TypedCallExt, TypedClassExt, TypedFieldAccessExt, TypedStringExt, builtin::JavaString, core::JNIEnv,
    define_java_class,
};
define_java_class!(JavaExample1, "org.example.Example1");
define_java_class!(JavaExample2, "org.example.Example2");
fn run_jni<'env>(env: &'env JNIEnv<'static>) {
    // create Example object
    let example1_cls: LocalClass<JavaExample1> = env.typed_find_class().unwrap();
    let example1_obj: LocalObject<JavaExample1> = env
        .typed_new_object(&example1_cls, (env.typed_new_string("Hello World!"),))
        .unwrap();
    // call hello method
    env.typed_call_method::<(), _, _>(&example1_obj, "hello", ()).unwrap();
    // get str field
    let str: LocalObject<JavaString> = env.typed_get_field(&example1_obj, "str").unwrap();
    let str: String = env.typed_get_string(&str);
    // create Example2 object
    let example2_cls: LocalClass<JavaExample2> = env.typed_find_class().unwrap();
    let example2_obj: LocalObject<JavaExample2> = env.typed_new_object(&example2_cls, (example1_obj,)).unwrap();
    // call staticHello method
    env.typed_call_method::<(), _, _>(&example2_cls, "staticHello", (3i32,))
        .unwrap();
    // call hello method
    env.typed_call_method::<(), _, _>(&example2_obj, "hello", (3i32,)).unwrap();
    // get staticExample1 field
    let static_example1: LocalObject<JavaExample1> = env.typed_get_field(&example2_cls, "staticExample1").unwrap();
    // get example1 field
    let example1: LocalObject<JavaExample1> = env.typed_get_field(&example2_obj, "example1").unwrap();
}
 ```

### Access Rust in Java

You should define native methods in Rust code with following signature.

 ```rust
  use typed_jni::{define_java_class, TrampolineObject, core::JNIEnv, builtin::JavaString, TypedStringExt};
define_java_class!(JavaExample1, "org.example.Example1");
#[unsafe(no_mangle)]
pub extern "system" fn Java_org_example_Example1_nativeHello<'env>(env: &'env JNIEnv, obj: TrampolineObject<'env, JavaExample1>, str: TrampolineObject<'env, JavaString>) {
    let str: String = env.typed_get_string(&str);
    println!("{}", str);
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_org_example_Example1_getStr<'env>(env: &'env JNIEnv, obj: TrampolineObject<'env, JavaExample1>) -> TrampolineObject<'env, JavaString> {
    env.typed_new_string("native string").into_trampoline()
}
 ```

Then load it in Java code.

 ```java
 System.loadLibrary("example1");
 ```

**NOTE**: All object reference in native function **parameters** and **return value** should be `TrampolineObject` or
`TrampolineClass`, it is ffi safe.

## Features

* `std` - Enables the use standard library. (default)
* `cache` - Enables the use cache for class and member lookups. (default, requires `std`)
