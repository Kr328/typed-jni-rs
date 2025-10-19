use typed_jni::{
    LocalClass, LocalObject, TrampolineClass, TrampolineObject, TypedCallExt, TypedClassExt, TypedStringExt,
    builtin::JavaString,
    core::{JNIEnv, NativeFunction},
    define_java_class,
};

use crate::{compile_file_and_load_classes, with_java_vm};

#[test]
fn test_register_native() {
    with_java_vm(|env| {
        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "RustNativeTest",
            r#"
                public class RustNativeTest {
                    private static native int nativeCall(int value, float value2, String value3);

                    public static int callNative(int value, float value2, String value3) {
                        return nativeCall(value, value2, value3);
                    }
                }
            "#,
        );

        define_java_class!(JavaRustNativeTest, "RustNativeTest");

        extern "system" fn call_native<'env>(
            env: &'env JNIEnv<'static>,
            _: TrampolineClass<'env, JavaRustNativeTest>,
            value: i32,
            value2: f32,
            value3: TrampolineObject<'env, JavaString>,
        ) -> i32 {
            let v = value + value2 as i32 + env.typed_get_string(&value3).len() as i32;

            v
        }

        let c_test: LocalClass<JavaRustNativeTest> = env.typed_find_class_in_class_loader(&loader).unwrap();

        unsafe {
            env.register_natives(
                &*c_test,
                [NativeFunction {
                    name: c"nativeCall",
                    signature: c"(IFLjava/lang/String;)I",
                    fn_ptr: call_native as *const (),
                }],
            )
            .unwrap()
        }

        assert_eq!(
            env.typed_call_method::<i32, _, _>(&c_test, "callNative", (114514, 12.78f32, env.typed_new_string("114514")))
                .unwrap(),
            114514 + 12 + 6
        );
    });
}

#[test]
fn test_return_object() {
    with_java_vm(|env| {
        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "RustNativeTest",
            r#"
                public class RustNativeTest {
                    private static native String nativeCall(boolean empty);

                    public static String callNative(boolean empty) {
                        return nativeCall(empty);
                    }
                }
            "#,
        );

        define_java_class!(JavaRustNativeTest, "RustNativeTest");

        extern "system" fn call_native<'env>(
            env: &'env JNIEnv,
            _: TrampolineClass<'env, JavaRustNativeTest>,
            empty: bool,
        ) -> Option<TrampolineObject<'env, JavaString>> {
            if empty {
                None
            } else {
                Some(env.typed_new_string("Some").into_trampoline())
            }
        }

        let c_test: LocalClass<JavaRustNativeTest> = env.typed_find_class_in_class_loader(&loader).unwrap();

        unsafe {
            env.register_natives(
                &*c_test,
                [NativeFunction {
                    name: c"nativeCall",
                    signature: c"(Z)Ljava/lang/String;",
                    fn_ptr: call_native as *const (),
                }],
            )
            .unwrap()
        }

        assert_eq!(
            env.typed_call_method::<Option<LocalObject<JavaString>>, _, _>(&c_test, "callNative", (false,))
                .unwrap()
                .map(|s| env.typed_get_string(&s))
                .as_deref(),
            Some("Some")
        );
        assert_eq!(
            env.typed_call_method::<Option<LocalObject<JavaString>>, _, _>(&c_test, "callNative", (true,))
                .unwrap()
                .map(|s| env.typed_get_string(&s)),
            None
        );
    })
}
