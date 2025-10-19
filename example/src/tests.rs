use std::{process::Stdio, ptr, sync::OnceLock};

use jni::JavaVM;
use typed_jni::{
    Array, LocalClass, LocalObject, TrampolineClass, TrampolineObject, TypedArrayExt, TypedCallExt, TypedClassExt,
    TypedFieldAccessExt, TypedObjectArrayExt, TypedPrimitiveArrayExt, TypedRef, TypedStringExt,
    builtin::{JavaClass, JavaString},
    core::{JNIEnv, NativeFunction},
    define_java_class,
};

fn with_java_vm<R, F: FnOnce(&JNIEnv) -> R>(f: F) -> R {
    static VM: OnceLock<JavaVM> = OnceLock::new();
    let vm = VM.get_or_init(|| JavaVM::new(jni::InitArgsBuilder::new().build().unwrap()).unwrap());
    let env = vm.attach_current_thread().unwrap();

    // temp workaround for jni crate not match jni-sys
    f(unsafe { JNIEnv::from_raw(env.get_raw() as _) })
}

#[test]
fn test_create_vm() {
    with_java_vm(|_| {
        println!("CRATED");
    })
}

#[test]
fn test_convert_string() {
    with_java_vm(|env| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                const TEST_CONTENT_URL: &str = "https://www.cogsci.ed.ac.uk/~richard/unicode-sample-3-2.html";

                let response = reqwest::get(TEST_CONTENT_URL).await.unwrap();
                let content = response.text().await.unwrap();

                let o_string: LocalObject<JavaString> = env.typed_new_string(&content);
                let r_content: String = env.typed_get_string(&o_string);

                assert_eq!(content, r_content);
            })
    })
}

#[test]
fn test_string_array() {
    with_java_vm(|env| {
        let length = rand::random::<u32>() % 128;
        let array = (0..length)
            .map(|_| {
                let length = rand::random::<u32>() % 128;
                (0..length).map(|_| rand::random::<char>()).collect::<String>()
            })
            .collect::<Vec<_>>();

        let o_array: LocalObject<Array<JavaString>> = env
            .typed_new_array(&env.typed_find_class().unwrap(), array.len() as _)
            .unwrap();
        for (index, s) in array.iter().enumerate() {
            env.typed_set_array_element(&o_array, index as _, Some(&env.typed_new_string(s)))
                .unwrap();
        }

        let r_length = env.typed_get_array_length(&o_array).unwrap();
        let mut r_array = Vec::with_capacity(r_length as _);
        for index in 0..r_length {
            let s: Option<LocalObject<JavaString>> = env.typed_get_array_element(&o_array, index).unwrap();

            r_array.push(env.typed_get_string(&s.unwrap()));
        }

        assert_eq!(array, r_array);
    })
}

#[test]
fn test_bool_array() {
    with_java_vm(|env| {
        let length = rand::random::<u32>() % 128;
        let array: Vec<bool> = (0..length).map(|_| rand::random::<bool>()).collect();

        let o_array: LocalObject<Array<bool>> = env.typed_new_primitive_array(array.len() as _).unwrap();
        env.typed_set_array_region(&o_array, 0, &array).unwrap();

        let mut r_array = vec![false; array.len()];
        env.typed_get_array_region(&o_array, 0, &mut r_array).unwrap();

        assert_eq!(array, r_array);
    })
}

define_java_class!(JavaFile, "java.io.File");
define_java_class!(JavaURI, "java.net.URI");
define_java_class!(JavaURL, "java.net.URL");
define_java_class!(JavaURLClassLoader, "java.net.URLClassLoader");

struct UrlClassLoader<'env> {
    _class_path: tempdir::TempDir,
    loader: LocalObject<'env, JavaURLClassLoader>,
}

fn compile_file_and_load_classes<'env>(env: &'env JNIEnv, public_class_name: &str, content: &str) -> UrlClassLoader<'env> {
    let temp = tempdir::TempDir::new("classes").unwrap();
    let file = temp.path().join(public_class_name).with_extension("java");

    std::fs::write(&file, content).unwrap();

    let javac_ret = std::process::Command::new("javac")
        .arg("-J-Duser.language=en")
        .arg(file.file_name().unwrap().to_str().unwrap())
        .current_dir(temp.path())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    if !javac_ret.success() {
        panic!("compile java failed");
    }

    let c_file: LocalClass<JavaFile> = env.typed_find_class().unwrap();
    let o_file: LocalObject<JavaFile> = env
        .typed_new_object(&c_file, (&env.typed_new_string(temp.path().to_str().unwrap()),))
        .unwrap();
    let o_uri: LocalObject<JavaURI> = env.typed_call_method(&o_file, "toURI", ()).unwrap();
    let o_url: LocalObject<JavaURL> = env.typed_call_method(&o_uri, "toURL", ()).unwrap();
    let o_urls: LocalObject<Array<JavaURL>> = env
        .typed_new_array_with_initial(&env.typed_find_class().unwrap(), 1, &o_url)
        .unwrap();
    let o_class_loader: LocalObject<JavaURLClassLoader> =
        env.typed_new_object(&env.typed_find_class().unwrap(), (&o_urls,)).unwrap();

    UrlClassLoader {
        _class_path: temp,
        loader: o_class_loader,
    }
}

#[test]
fn test_inner_class() {
    with_java_vm(|env| {
        let loader = compile_file_and_load_classes(
            env,
            "RustTest",
            r#"
                public class RustTest {
                    static final InnerClass INNER = new InnerClass();

                    public static class InnerClass {
                        public final String VALUE = "STRING FROM INNER CLASS";
                    }
                }
            "#,
        );

        define_java_class!(JavaRustTest, "RustTest");
        define_java_class!(JavaInnerClass, "RustTest$InnerClass");

        let c_test: LocalObject<JavaClass> = env
            .typed_call_method(&loader.loader, "loadClass", (&env.typed_new_string("RustTest"),))
            .unwrap();
        let c_test: LocalClass<JavaRustTest> = unsafe { LocalClass::from_ref(c_test.into_ref()) };

        let o_inner: LocalObject<JavaInnerClass> = env.typed_get_field(&c_test, "INNER").unwrap();
        let value: LocalObject<JavaString> = env.typed_get_field(&o_inner, "VALUE").unwrap();

        assert_eq!("STRING FROM INNER CLASS", env.typed_get_string(&value));
    });
}

#[test]
fn test_register_native() {
    with_java_vm(|env| {
        let loader = compile_file_and_load_classes(
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

        extern "C" fn call_native<'env>(
            env: &'env JNIEnv<'static>,
            _: TrampolineClass<'env, JavaRustNativeTest>,
            value: i32,
            value2: f32,
            value3: TrampolineObject<'env, JavaString>,
        ) -> i32 {
            let v = value + value2 as i32 + env.typed_get_string(&value3).len() as i32;

            v
        }

        let c_test: LocalObject<JavaClass> = env
            .typed_call_method(&loader.loader, "loadClass", (&env.typed_new_string("RustNativeTest"),))
            .unwrap();
        let c_test: LocalClass<JavaRustNativeTest> = unsafe { LocalClass::from_ref(c_test.into_ref()) };

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
fn test_boolean_parameter() {
    with_java_vm(|env| {
        define_java_class!(JavaAtomicBoolean, "java.util.concurrent.atomic.AtomicBoolean");

        let c_atomic_boolean: LocalClass<JavaAtomicBoolean> = env.typed_find_class().unwrap();
        let o_atomic_boolean = env.typed_new_object(&c_atomic_boolean, (true,)).unwrap();

        let success: bool = env
            .typed_call_method(&o_atomic_boolean, "compareAndSet", (true, false))
            .unwrap();
        assert!(success);

        let success: bool = env
            .typed_call_method(&o_atomic_boolean, "compareAndSet", (true, false))
            .unwrap();
        assert!(!success);
    });
}

#[test]
fn test_find_array_class() {
    with_java_vm(|env| {
        env.typed_find_class::<Array<bool>>().unwrap();
        env.typed_find_class::<Array<JavaString>>().unwrap();
        env.typed_find_class::<Array<Array<bool>>>().unwrap();
        env.typed_find_class::<Array<Array<JavaString>>>().unwrap();
    })
}

#[test]
fn test_return_object() {
    with_java_vm(|env| {
        let loader = compile_file_and_load_classes(
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

        extern "C" fn call_native<'env>(
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

        let c_test: LocalObject<JavaClass> = env
            .typed_call_method(&loader.loader, "loadClass", (&env.typed_new_string("RustNativeTest"),))
            .unwrap();
        let c_test: LocalClass<JavaRustNativeTest> = unsafe { LocalClass::from_ref(c_test.into_ref()) };

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

#[test]
fn test_drop_after_consume() {
    struct Struct<'a> {
        mark: &'a mut bool,
        ptr: *const (),
    }

    impl<'a> Drop for Struct<'a> {
        fn drop(&mut self) {
            *self.mark = true;
        }
    }

    impl<'a> Struct<'a> {
        fn consume(self) {
            let _ = self.ptr;

            std::mem::forget(self);
        }
    }

    let mut dropped = false;

    let s = Struct {
        mark: &mut dropped,
        ptr: ptr::null_mut(),
    };

    drop(s);

    assert!(dropped);

    let mut dropped = false;

    let s = Struct {
        mark: &mut dropped,
        ptr: ptr::null_mut(),
    };

    s.consume();

    assert!(!dropped);
}

#[test]
fn test_int_array_access() {
    with_java_vm(|env| {
        let array = env.typed_new_primitive_array::<i32>(8).unwrap();

        let mut elements = env.typed_get_array_elements(&array).unwrap();

        elements[0] = 1;
        elements[1] = 2;
        elements[2] = 3;
        elements[3] = 4;

        elements.commit();

        let mut buf = [0i32; 8];

        env.typed_get_array_region(&array, 0, &mut buf[..]).unwrap();

        assert_eq!(buf, [1, 2, 3, 4, 0, 0, 0, 0]);

        let mut elements = env.typed_get_array_elements(&array).unwrap();

        elements[4] = 1;
        elements[5] = 2;
        elements[6] = 3;
        elements[7] = 4;

        drop(elements);

        let mut buf = [0i32; 8];

        env.typed_get_array_region(&array, 0, &mut buf).unwrap();

        assert_eq!(buf, [1, 2, 3, 4, 0, 0, 0, 0]);

        env.typed_set_array_region(&array, 4, &[8, 9, 10, 11]).unwrap();

        let buf = env.typed_get_array_elements(&array).unwrap();

        assert_eq!(*buf, [1, 2, 3, 4, 8, 9, 10, 11])
    })
}

#[test]
fn test_bytes_access() {
    let s = "Hello你好こんにちは안녕하세요";

    with_java_vm(|env| {
        let array: LocalObject<Array<i8>> = env.typed_new_primitive_array::<i8>(s.as_bytes().len() as _).unwrap();

        let mut elements = env.typed_get_bytes_array_elements(&array).unwrap();

        elements.copy_from_slice(s.as_bytes());

        elements.commit();

        let java_s: LocalClass<JavaString> = env.typed_find_class::<JavaString>().unwrap();
        let java_s: LocalObject<JavaString> = env.typed_new_object(&java_s, (&array,)).unwrap();
        assert_eq!(env.typed_get_string(&java_s), s);

        let array: LocalObject<Array<i8>> = env.typed_call_method(&java_s, "getBytes", ()).unwrap();
        assert_eq!(&*env.typed_get_bytes_array_elements(&array).unwrap(), s.as_bytes());
    })
}
