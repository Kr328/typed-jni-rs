#[cfg(test)]
mod tests;

use std::{path::Path, process::Stdio};

use example::JavaExample;
use jni::JavaVM;
use typed_jni::{define_java_class, Class, Context, JString, Object};

define_java_class!(JavaSystem, "java/lang/System");
define_java_class!(JavaPrintStream, "java/io/PrintStream");

fn main() {
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

    typed_jni::attach_vm(vm.get_java_vm_pointer() as _);

    let env = vm.attach_current_thread().unwrap();
    let ctx = unsafe { Context::from_raw(env.get_raw() as _) };

    unsafe {
        let obj = ctx.find_class(c"java/lang/Object").unwrap();
        let str = ctx.find_class(c"java/lang/String").unwrap();

        println!("{}", ctx.is_assignable_from(&str, &obj));

        let c_str = ctx.get_object_class(&str);
        drop(c_str);
    }

    let c_system = Class::<JavaSystem>::find_class(&ctx).unwrap();
    let o_out: Object<JavaPrintStream> = Option::unwrap(c_system.get_field(&ctx, "out").unwrap());

    let _: () = o_out
        .call_method(&ctx, "println", &Object::<JString>::new_string(&ctx, "Hello World!"))
        .unwrap();

    let _: () = o_out
        .call_method(&ctx, "println", &Object::<JString>::new_string(&ctx, "Hello World!!!!"))
        .unwrap();

    let _: () = o_out
        .call_method(&ctx, "println", &Object::<JString>::new_string(&ctx, "Hello World!!!!!!!!!"))
        .unwrap();

    let v = Class::<JavaExample>::find_class(ctx).unwrap();

    let _: () = v.call_method(&ctx, "run", ()).unwrap();
}
