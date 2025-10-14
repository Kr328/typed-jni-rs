use std::{process::Stdio, ptr, sync::OnceLock};

use jni::JavaVM;
use typed_jni::{
    Array, AsRaw, Class, Context, JavaString, LocalClass, LocalObject, NoArgs, Object, TrampolineClass, TrampolineObject,
    define_java_class,
};

fn with_java_vm<R, F: FnOnce(&Context) -> R>(f: F) -> R {
    static VM: OnceLock<JavaVM> = OnceLock::new();
    let vm = VM.get_or_init(|| JavaVM::new(jni::InitArgsBuilder::new().build().unwrap()).unwrap());
    let env = vm.attach_current_thread().unwrap();

    typed_jni::attach_vm(vm.get_java_vm_pointer() as _);

    // temp workaround for jni crate not match jni-sys
    f(unsafe { Context::from_raw(env.get_raw() as _) })
}

#[test]
fn test_create_vm() {
    with_java_vm(|_| {
        println!("CRATED");
    })
}

#[test]
fn test_convert_string() {
    with_java_vm(|ctx| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                const TEST_CONTENT_URL: &str = "https://www.cogsci.ed.ac.uk/~richard/unicode-sample-3-2.html";

                let response = reqwest::get(TEST_CONTENT_URL).await.unwrap();
                let content = response.text().await.unwrap();

                let o_string: LocalObject<JavaString> = Object::new_string(ctx, &content);
                let r_content: String = o_string.get_string(ctx);

                assert_eq!(content, r_content);
            })
    })
}

#[test]
fn test_string_array() {
    with_java_vm(|ctx| {
        let length = rand::random::<u32>() % 128;
        let array = (0..length)
            .map(|_| {
                let length = rand::random::<u32>() % 128;
                (0..length).map(|_| rand::random::<char>()).collect::<String>()
            })
            .collect::<Vec<_>>();

        let o_array =
            LocalObject::<Array<JavaString>>::new(ctx, array.len() as _, &LocalClass::<JavaString>::find_class(ctx).unwrap())
                .unwrap();
        for (index, s) in array.iter().enumerate() {
            o_array
                .set_element(ctx, index as _, Some(&LocalObject::<JavaString>::new_string(ctx, s)))
                .unwrap();
        }

        let r_length = o_array.length(ctx);
        let mut r_array = Vec::with_capacity(r_length as _);
        for index in 0..r_length {
            let s: Option<LocalObject<JavaString>> = o_array.get_element(ctx, index).unwrap();

            r_array.push(s.unwrap().get_string(ctx));
        }

        assert_eq!(array, r_array);
    })
}

#[test]
fn test_bool_array() {
    with_java_vm(|ctx| {
        let length = rand::random::<u32>() % 128;
        let array: Vec<bool> = (0..length).map(|_| rand::random::<bool>()).collect();

        let o_array = LocalObject::<Array<bool>>::new_primitive(ctx, array.len() as _).unwrap();
        o_array.set_region(ctx, 0, &array).unwrap();

        let mut r_array = vec![false; array.len()];
        o_array.get_region(ctx, 0, &mut r_array).unwrap();

        assert_eq!(array, r_array);
    })
}

define_java_class!(JavaFile, "java.io.File");
define_java_class!(JavaURI, "java.net.URI");
define_java_class!(JavaURL, "java.net.URL");
define_java_class!(JavaURLClassLoader, "java.net.URLClassLoader");

struct UrlClassLoader<'ctx> {
    _class_path: tempdir::TempDir,
    loader: LocalObject<'ctx, JavaURLClassLoader>,
}

fn compile_file_and_load_classes<'ctx>(ctx: &'ctx Context, public_class_name: &str, content: &str) -> UrlClassLoader<'ctx> {
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

    let c_file: LocalClass<JavaFile> = Class::find_class(ctx).unwrap();
    let o_file: LocalObject<JavaFile> = c_file
        .new_object(
            ctx,
            &LocalObject::<JavaString>::new_string(ctx, temp.path().to_str().unwrap()),
        )
        .unwrap();
    let o_uri: LocalObject<JavaURI> = o_file.call_method(ctx, "toURI", NoArgs).unwrap();
    let o_url: LocalObject<JavaURL> = o_uri.call_method(ctx, "toURL", NoArgs).unwrap();
    let o_urls: LocalObject<Array<JavaURL>> = Object::new_with_initial(ctx, 1, &Class::find_class(ctx).unwrap(), &o_url).unwrap();
    let o_class_loader = LocalClass::<JavaURLClassLoader>::find_class(ctx)
        .unwrap()
        .new_object(ctx, &o_urls)
        .unwrap();

    UrlClassLoader {
        _class_path: temp,
        loader: o_class_loader,
    }
}

#[test]
fn test_inner_class() {
    with_java_vm(|ctx| {
        let loader = compile_file_and_load_classes(
            ctx,
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

        let c_test: LocalClass<JavaRustTest> = loader
            .loader
            .call_method(ctx, "loadClass", &LocalObject::<JavaString>::new_string(ctx, "RustTest"))
            .unwrap();

        let o_inner: LocalObject<JavaInnerClass> = c_test.get_field(ctx, "INNER").unwrap();
        let value: LocalObject<JavaString> = o_inner.get_field(ctx, "VALUE").unwrap();

        assert_eq!("STRING FROM INNER CLASS", value.get_string(ctx));
    });
}

#[test]
fn test_register_native() {
    with_java_vm(|ctx| {
        let loader = compile_file_and_load_classes(
            ctx,
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

        extern "C" fn call_native<'ctx>(
            ctx: &'ctx Context,
            _: TrampolineClass<'ctx, JavaRustNativeTest>,
            value: i32,
            value2: f32,
            value3: TrampolineObject<'ctx, JavaString>,
        ) -> i32 {
            let v = value + value2 as i32 + value3.get_string(ctx).len() as i32;

            v
        }

        let c_test: LocalClass<JavaRustNativeTest> = loader
            .loader
            .call_method(ctx, "loadClass", &Object::new_string(ctx, "RustNativeTest"))
            .unwrap();

        unsafe {
            ctx.register_natives(
                c_test.as_raw(),
                [(c"nativeCall", c"(IFLjava/lang/String;)I", call_native as *const ())],
            )
            .unwrap()
        }

        assert_eq!(
            c_test
                .call_method::<i32, _>(
                    ctx,
                    "callNative",
                    (114514, 12.78f32, &LocalObject::<JavaString>::new_string(ctx, "114514"))
                )
                .unwrap(),
            114514 + 12 + 6
        );
    });
}

#[test]
fn test_boolean_parameter() {
    with_java_vm(|ctx| {
        define_java_class!(JavaAtomicBoolean, "java.util.concurrent.atomic.AtomicBoolean");

        let c_atomic_boolean = LocalClass::<JavaAtomicBoolean>::find_class(ctx).unwrap();
        let o_atomic_boolean = c_atomic_boolean.new_object(ctx, true).unwrap();

        let success: bool = o_atomic_boolean.call_method(ctx, "compareAndSet", (true, false)).unwrap();
        assert!(success);

        let success: bool = o_atomic_boolean.call_method(ctx, "compareAndSet", (true, false)).unwrap();
        assert!(!success);
    });
}

#[test]
fn test_find_array_class() {
    with_java_vm(|ctx| {
        LocalClass::<Array<bool>>::find_class(ctx).unwrap();
        LocalClass::<Array<JavaString>>::find_class(ctx).unwrap();
        LocalClass::<Array<Array<bool>>>::find_class(ctx).unwrap();
        LocalClass::<Array<Array<JavaString>>>::find_class(ctx).unwrap();
    })
}

#[test]
fn test_return_object() {
    with_java_vm(|ctx| {
        let loader = compile_file_and_load_classes(
            ctx,
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

        extern "C" fn call_native<'ctx>(
            ctx: &'ctx Context,
            _: TrampolineClass<'ctx, JavaRustNativeTest>,
            empty: bool,
        ) -> Option<LocalObject<'ctx, JavaString>> {
            if empty { None } else { Some(Object::new_string(ctx, "Some")) }
        }

        let c_test: LocalClass<JavaRustNativeTest> = loader
            .loader
            .call_method(ctx, "loadClass", &Object::new_string(ctx, "RustNativeTest"))
            .unwrap();

        unsafe {
            ctx.register_natives(
                c_test.as_raw(),
                [(c"nativeCall", c"(Z)Ljava/lang/String;", call_native as *const ())],
            )
            .unwrap()
        }

        assert_eq!(
            c_test
                .call_method::<Option<LocalObject<JavaString>>, _>(ctx, "callNative", false)
                .unwrap()
                .map(|s| s.get_string(ctx))
                .as_deref(),
            Some("Some")
        );
        assert_eq!(
            c_test
                .call_method::<Option<LocalObject<JavaString>>, _>(ctx, "callNative", true)
                .unwrap()
                .map(|s| s.get_string(ctx)),
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
    with_java_vm(|ctx| {
        let array = LocalObject::<Array<i32>>::new_primitive(ctx, 8).unwrap();

        let mut elements = array.get_elements(ctx);

        elements[0] = 1;
        elements[1] = 2;
        elements[2] = 3;
        elements[3] = 4;

        elements.commit();

        let mut buf = [0i32; 8];

        array.get_region(ctx, 0, &mut buf[..]).unwrap();

        assert_eq!(buf, [1, 2, 3, 4, 0, 0, 0, 0]);

        let mut elements = array.get_elements(ctx);

        elements[4] = 1;
        elements[5] = 2;
        elements[6] = 3;
        elements[7] = 4;

        drop(elements);

        let mut buf = [0i32; 8];

        array.get_region(ctx, 0, &mut buf).unwrap();

        assert_eq!(buf, [1, 2, 3, 4, 0, 0, 0, 0]);

        array.set_region(ctx, 4, &[8, 9, 10, 11]).unwrap();

        let buf = array.get_elements(ctx);

        assert_eq!(*buf, [1, 2, 3, 4, 8, 9, 10, 11])
    })
}

#[test]
fn test_bytes_access() {
    let s = "Hello你好こんにちは안녕하세요";

    with_java_vm(|ctx| {
        let array = LocalObject::<Array<i8>>::new_primitive(ctx, s.as_bytes().len() as _).unwrap();

        let mut elements = array.get_bytes_elements(ctx);

        elements.copy_from_slice(s.as_bytes());

        elements.commit();

        let java_s = LocalClass::<JavaString>::find_class(ctx).unwrap();
        let java_s = java_s.new_object(ctx, &array).unwrap();
        assert_eq!(java_s.get_string(ctx).as_str(), s);

        let array: LocalObject<Array<i8>> = java_s.call_method(ctx, "getBytes", NoArgs).unwrap();
        assert_eq!(&*array.get_bytes_elements(ctx), s.as_bytes());
    })
}
