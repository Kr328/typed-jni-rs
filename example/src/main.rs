use std::{path::Path, process::Stdio};

use example::JavaExample;
use jni::JavaVM;
use typed_jni::{Context, JavaString, LocalClass, LocalObject, NoArgs, Type, define_java_class};

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

    let c_system = LocalClass::<JavaSystem>::find_class(&ctx).unwrap();
    let o_out: LocalObject<JavaPrintStream> = c_system.get_field(&ctx, "out").unwrap();

    o_out
        .call_method::<(), _>(&ctx, "println", &LocalObject::<JavaString>::new_string(&ctx, "Hello World!"))
        .unwrap();

    o_out
        .call_method::<(), _>(
            &ctx,
            "println",
            &LocalObject::<JavaString>::new_string(&ctx, "Hello World!!!!"),
        )
        .unwrap();

    o_out
        .call_method::<(), _>(
            &ctx,
            "println",
            &LocalObject::<JavaString>::new_string(&ctx, "Hello World!!!!!!!!!"),
        )
        .unwrap();

    let v = LocalClass::<JavaExample>::find_class(ctx).unwrap();

    let _: () = v.call_method(&ctx, "run", NoArgs).unwrap();
}
