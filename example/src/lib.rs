use typed_jni::{TrampolineClass, TrampolineObject, TypedStringExt, builtin::JavaString, core::JNIEnv, define_java_class};

define_java_class!(JavaExample, "com.github.kr328.typedjni.Example");

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_github_kr328_typedjni_Example_nativeFunction<'ctx>(
    env: &'ctx JNIEnv<'static>,
    _class: TrampolineClass<'ctx, JavaExample>,
    value: i32,
    value2: f32,
    value3: TrampolineObject<'ctx, JavaString>,
) {
    println!("value = {}", value);
    println!("value2 = {}", value2);
    println!("value3 = {}", env.typed_get_string(&value3));
}
