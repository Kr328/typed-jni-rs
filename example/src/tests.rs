use std::{process::Stdio, sync::OnceLock};

use jni::JavaVM;
use typed_jni::{
    define_java_class, CallMethod, Context, FindClass, GetField, Global, JavaArray, JavaPrimitiveArray, JavaString, Local,
    NewObject, This,
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
pub fn test_create_vm() {
    with_java_vm(|_| {
        println!("CRATED");
    })
}

#[test]
pub fn test_convert_string() {
    with_java_vm(|ctx| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                const TEST_CONTENT_URL: &str = "https://www.cogsci.ed.ac.uk/~richard/unicode-sample-3-2.html";

                let response = reqwest::get(TEST_CONTENT_URL).await.unwrap();
                let content = response.text().await.unwrap();

                let o_string: JavaString<false, Local> = JavaString::from_string(ctx, &content);
                let r_content: String = o_string.to_string(ctx);

                assert_eq!(content, r_content);
            })
    })
}

#[test]
pub fn test_string_array() {
    with_java_vm(|ctx| {
        let length = rand::random::<usize>() % 128;
        let array = (0..length)
            .map(|_| {
                let length = rand::random::<usize>() % 128;
                (0..length).map(|_| rand::random::<char>()).collect::<String>()
            })
            .collect::<Vec<_>>();

        let o_array = JavaArray::new::<Global>(ctx, array.len() as _, &JavaString::find_class(ctx).unwrap(), None).unwrap();
        for (index, s) in array.iter().enumerate() {
            o_array
                .set_element(ctx, index as _, Some(&JavaString::from_string(ctx, s)))
                .unwrap();
        }

        let r_length = o_array.length(ctx);
        let mut r_array = Vec::with_capacity(r_length as _);
        for index in 0..r_length {
            let s: Option<JavaString<false, Local>> = o_array.get_element(ctx, index).unwrap();

            r_array.push(s.unwrap().to_string(ctx));
        }

        assert_eq!(array, r_array);
    })
}

#[test]
pub fn test_bool_array() {
    with_java_vm(|ctx| {
        let length: usize = rand::random::<usize>() % 128;
        let array: Vec<bool> = (0..length).map(|_| rand::random::<bool>()).collect();

        let o_array = JavaPrimitiveArray::new(ctx, array.len() as _).unwrap();
        o_array.set_region(ctx, 0, &array).unwrap();

        let mut r_array = vec![false; array.len()];
        o_array.get_region(ctx, 0, &mut r_array).unwrap();

        assert_eq!(array, r_array);
    })
}

define_java_class!(JavaFile, "java/io/File");
define_java_class!(JavaURI, "java/net/URI");
define_java_class!(JavaURL, "java/net/URL");
define_java_class!(JavaURLClassLoader, "java/net/URLClassLoader");

struct UrlClassLoader<'ctx> {
    _class_path: tempdir::TempDir,
    loader: JavaURLClassLoader<false, Local<'ctx>>,
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

    let c_file: JavaFile<true, Local> = JavaFile::find_class(ctx).unwrap();
    let o_file: JavaFile<false, Local> = c_file
        .new_object(ctx, &JavaString::from_string(ctx, temp.path().to_str().unwrap()))
        .unwrap();
    let o_uri: JavaURI<false, Local> = Option::unwrap(o_file.call_method(ctx, "toURI", ()).unwrap());
    let o_url: JavaURL<false, Local> = Option::unwrap(o_uri.call_method(ctx, "toURL", ()).unwrap());
    let o_urls: JavaArray<false, JavaURL<true, Local>, Local> =
        JavaArray::new(ctx, 1, &JavaURL::find_class(ctx).unwrap(), Some(&o_url)).unwrap();
    let o_class_loader = JavaURLClassLoader::find_class(ctx).unwrap().new_object(ctx, &o_urls).unwrap();

    UrlClassLoader {
        _class_path: temp,
        loader: o_class_loader,
    }
}

#[test]
pub fn test_inner_class() {
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

        let c_test: JavaRustTest<true, _> = Option::unwrap(
            loader
                .loader
                .call_method(ctx, "loadClass", &JavaString::from_string(ctx, "RustTest"))
                .unwrap(),
        );

        let o_inner: JavaInnerClass<false, _> = Option::unwrap(c_test.get_field(ctx, "INNER").unwrap());
        let value: JavaString<false, _> = Option::unwrap(o_inner.get_field(ctx, "VALUE").unwrap());

        assert_eq!("STRING FROM INNER CLASS", value.to_string(ctx));
    });
}

#[test]
pub fn test_register_native() {
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
            _class: JavaRustNativeTest<false, Local<'ctx>>,
            value: i32,
            value2: f32,
            value3: JavaString<false, Local<'ctx>>,
        ) -> i32 {
            let v = value + value2 as i32 + value3.to_string(ctx).len() as i32;

            v
        }

        let c_test: JavaRustNativeTest<true, _> = Option::unwrap(
            loader
                .loader
                .call_method(ctx, "loadClass", &JavaString::from_string(ctx, "RustNativeTest"))
                .unwrap(),
        );

        unsafe {
            ctx.register_natives(
                c_test.as_ref(),
                [(c"nativeCall", c"(IFLjava/lang/String;)I", call_native as *const ())],
            )
            .unwrap()
        }

        assert_eq!(
            c_test
                .call_method::<3, i32, _>(ctx, "callNative", (114514, 12.78f32, &JavaString::from_string(ctx, "114514")))
                .unwrap(),
            114514 + 12 + 6
        );
        // assert_eq!(b_test.native_call::<jint>(env, c_test, 1919810).unwrap(), 1919811);
    });
}

#[test]
pub fn test_boolean_parameter() {
    with_java_vm(|ctx| {
        define_java_class!(JavaAtomicBoolean, "java/util/concurrent/atomic/AtomicBoolean");

        let c_atomic_boolean = JavaAtomicBoolean::find_class(ctx).unwrap();
        let o_atomic_boolean: JavaAtomicBoolean<false, Local> = c_atomic_boolean.new_object(ctx, true).unwrap();

        let success: bool = o_atomic_boolean.call_method(ctx, "compareAndSet", (true, false)).unwrap();
        assert!(success);

        let success: bool = o_atomic_boolean.call_method(ctx, "compareAndSet", (true, false)).unwrap();
        assert!(!success);
    });
}
