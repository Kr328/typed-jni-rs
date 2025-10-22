use typed_jni::{
    LocalClass, LocalObject, TypedCallExt, TypedClassExt, TypedStringExt,
    builtin::JavaString,
    core::{JNIEnv, NativeFunction},
    define_java_class,
};

use crate::{compile_file_and_load_classes, with_java_vm};

#[test]
fn test_cache_on_java_created_thread() {
    with_java_vm(|env| {
        let (_dir, cl) = compile_file_and_load_classes(
            env,
            "Test",
            r#"
        public class Test {
            public static native void nativeFunction();

            public static void run() throws InterruptedException {
                Thread thread = new Thread(() -> {
                    nativeFunction();
                });

                thread.start();
                thread.join();
            }
        }
        "#,
        );

        extern "system" fn native_func(env: &JNIEnv) {
            let raw_s = "Hello, World!";

            let s: LocalObject<JavaString> = env.typed_new_string(raw_s);

            let len: i32 = env.typed_call_method(&s, "length", ()).unwrap();
            assert_eq!(len, raw_s.len() as i32);

            let is_empty: bool = env.typed_call_method(&s, "isEmpty", ()).unwrap();
            assert!(!is_empty);

            let world_pos: i32 = env
                .typed_call_method(&s, "indexOf", (env.typed_new_string("World"),))
                .unwrap();
            assert_eq!(world_pos, 7);
        }

        define_java_class!(JavaTest, "Test");

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&cl).unwrap();

        unsafe {
            env.register_natives(
                &*c_test,
                [NativeFunction {
                    name: c"nativeFunction",
                    signature: c"()V",
                    fn_ptr: native_func as *const (),
                }],
            )
            .unwrap();
        }

        env.typed_call_method::<(), _, _>(&c_test, "run", ()).unwrap();
    });
}
