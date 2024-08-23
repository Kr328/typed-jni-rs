use typed_jni::{
    define_java_class,
    sys::{jint, JavaVM, JNI_VERSION_1_6},
    Context, JavaString, Local,
};

define_java_class!(JavaExample, "com/github/kr328/typedjni/Example");

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_typedjni_Example_nativeFunction<'ctx>(
    ctx: &'ctx Context,
    _class: JavaExample<false, Local<'ctx>>,
    value: i32,
    value2: f32,
    value3: JavaString<false, Local<'ctx>>,
) {
    println!("value = {}", value);
    println!("value2 = {}", value2);
    println!("value3 = {}", value3.to_string(ctx));
}

#[no_mangle]
pub extern "C" fn JNI_OnLoad(vm: *mut JavaVM, _: *const ()) -> jint {
    typed_jni::attach_vm(vm);

    JNI_VERSION_1_6 as _
}