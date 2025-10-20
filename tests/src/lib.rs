#![cfg(test)]

mod array;
mod call;
mod class;
mod field;
mod native;
mod object;
mod string;

use std::{process::Stdio, sync::OnceLock};

use typed_jni::{
    Array, LocalClass, LocalObject, TypedCallExt, TypedClassExt, TypedObjectArrayExt, TypedObjectExt, TypedStringExt,
    builtin::JavaClassLoader,
    core::{JNIEnv, JavaVM},
    define_java_class,
};

fn with_java_vm<R, F: for<'env> FnOnce(&'env JNIEnv<'static>) -> R>(f: F) -> R {
    static VM: OnceLock<jni::JavaVM> = OnceLock::new();

    let vm = VM.get_or_init(|| jni::JavaVM::new(jni::InitArgsBuilder::new().build().unwrap()).unwrap());

    let vm: &'static JavaVM = unsafe { JavaVM::from_raw(vm.get_java_vm_pointer() as _) };

    vm.with_attached_thread(false, |env| f(env)).unwrap()
}

#[test]
fn test_create_vm() {
    with_java_vm(|_| {
        println!("CRATED");
    })
}

fn compile_file_and_load_classes<'env>(
    env: &'env JNIEnv,
    public_class_name: &str,
    content: &str,
) -> (tempdir::TempDir, LocalObject<'env, JavaClassLoader>) {
    define_java_class!(JavaFile, "java/io/File");
    define_java_class!(JavaURI, "java/net/URI");
    define_java_class!(JavaURL, "java/net/URL");
    define_java_class!(JavaURLClassLoader, "java/net/URLClassLoader");

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
    let c_class_loader: LocalClass<JavaClassLoader> = env.typed_find_class().unwrap();
    let o_class_loader: LocalObject<JavaURLClassLoader> =
        env.typed_new_object(&env.typed_find_class().unwrap(), (&o_urls,)).unwrap();

    (temp, env.typed_cast(&o_class_loader, &c_class_loader).unwrap())
}

#[test]
fn test_compile_file() {
    with_java_vm(|env| {
        define_java_class!(JavaTest, "Test");

        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "Test",
            r#"public class Test {
                public static boolean ready() {
                    return true;
                }
            }"#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();
        let ready: bool = env.typed_call_method(&c_test, "ready", ()).unwrap();
        assert!(ready);
    })
}
