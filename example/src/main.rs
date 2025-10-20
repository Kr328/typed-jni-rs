use std::{path::Path, process::Stdio};

use example::JavaExample;
use jni::JavaVM;
use typed_jni::{
    DynArg, LocalClass, LocalObject, Type, TypedCallExt, TypedClassExt, TypedFieldAccessExt, TypedStringExt, define_java_class,
};

define_java_class!(JavaSystem, "java.lang.System");
define_java_class!(JavaPrintStream, "java.io.PrintStream");

fn main() {
    println!("{}", JavaSystem::SIGNATURE);

    let classpath = Path::new("example").join("java");

    let ok = std::process::Command::new("javac")
        .args(["-h", "include", "com/github/kr328/typedjni/Example.java"])
        .current_dir(&classpath)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap()
        .status
        .success();
    if !ok {
        println!("compile java failed");

        return;
    }

    let library_path = std::env::current_exe().unwrap();

    let vm = JavaVM::new(
        jni::InitArgsBuilder::new()
            .option(format!(
                "-Djava.library.path={}",
                library_path.parent().unwrap().to_str().unwrap()
            ))
            .option(format!("-Djava.class.path={}", classpath.to_str().unwrap()))
            .build()
            .unwrap(),
    )
    .unwrap();

    let env = vm.attach_current_thread().unwrap();

    let env = unsafe { typed_jni::core::JNIEnv::from_raw(env.get_raw() as _) };

    let c_system: LocalClass<JavaSystem> = env.typed_find_class().unwrap();
    let o_out: LocalObject<JavaPrintStream> = env.typed_get_field(&c_system, "out").unwrap();

    env.typed_call_method::<(), _, _>(&o_out, "println", &[&env.typed_new_string("Hello World!") as &dyn DynArg])
        .unwrap();

    env.typed_call_method::<(), _, _>(&o_out, "println", (env.typed_new_string("Hello World!!!!"),))
        .unwrap();

    env.typed_call_method::<(), _, _>(&o_out, "println", (env.typed_new_string("Hello World!!!!!!!!!"),))
        .unwrap();

    let v: LocalClass<JavaExample> = env.typed_find_class().unwrap();

    let _: () = env.typed_call_method(&v, "run", ()).unwrap();
}
