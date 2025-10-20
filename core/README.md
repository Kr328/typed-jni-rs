# Basic JNI Bindings for Rust

[![crates.io](https://img.shields.io/crates/v/typed-jni-core.svg)](https://crates.io/crates/typed-jni-core)
[![docs.rs](https://img.shields.io/docsrs/typed-jni-core)](https://docs.rs/typed-jni-core)

This crate provides basic JNI bindings for Rust.

## Example

```rust
use typed_jni_core::{Arg, FieldID, JNIEnv, LocalRef, MethodID};

pub fn run_jni(env: &JNIEnv) {
    unsafe {
        let c_system = env.find_class(c"java/lang/System").unwrap();

        let f_out: FieldID<true> = env.get_field_id(&c_system, c"out", c"Ljava/io/PrintStream;").unwrap();
        let o_out: Option<LocalRef> = env.get_object_field(&c_system, f_out).unwrap();

        let c_print_stream = env.get_object_class(o_out.as_ref().unwrap());
        let m_println: MethodID<false> = env
            .get_method_id(&c_print_stream, c"println", c"(Ljava/lang/String;)V")
            .unwrap();

        let s_hello = env.new_string("Hello, World!");
        env.call_void_method(o_out.as_ref().unwrap(), m_println, [Arg::from(&s_hello)])
            .unwrap();
    }
}
```

## Features

- `alloc`: Enables the use of `alloc` crate for dynamic memory allocation. (default)
- `print-throwable`: Enables the printing of throwable objects.
