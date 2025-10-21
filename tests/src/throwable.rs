use typed_jni::{
    LocalClass, LocalObject, TypedCallExt, TypedClassExt, TypedObjectExt, TypedStringExt, TypedThrowableExt,
    builtin::JavaThrowable, define_java_class,
};

use crate::with_java_vm;

#[test]
fn test_run_with_pending_throwable() {
    with_java_vm(|env| {
        define_java_class!(JavaIllegalArgumentException, "java/lang/IllegalArgumentException");

        let c_throwable: LocalClass<JavaThrowable> = env.typed_find_class().unwrap();

        let c_exception: LocalClass<JavaIllegalArgumentException> = env.typed_find_class().unwrap();
        let o_exception: LocalObject<JavaIllegalArgumentException> = env.typed_new_object(&c_exception, ()).unwrap();

        env.typed_throw(&env.typed_cast(&o_exception, &c_throwable).unwrap());

        define_java_class!(JavaStringBuilder, "java/lang/StringBuilder");

        let c_string_builder: LocalClass<JavaStringBuilder> = env.typed_find_class().unwrap();
        let mut o_string_builder: LocalObject<JavaStringBuilder> = env.typed_new_object(&c_string_builder, ()).unwrap();

        o_string_builder = env
            .typed_call_method(&o_string_builder, "append", (env.typed_new_string("Hello, World!"),))
            .unwrap();
        o_string_builder = env.typed_call_method(&o_string_builder, "append", (1i32,)).unwrap();
        o_string_builder = env.typed_call_method(&o_string_builder, "append", (1i32,)).unwrap();
        o_string_builder = env.typed_call_method(&o_string_builder, "append", (4i64,)).unwrap();
        o_string_builder = env
            .typed_call_method(&o_string_builder, "append", (env.typed_new_string("514"),))
            .unwrap();

        let s = env.typed_to_string(&o_string_builder).unwrap();
        assert_eq!(s, "Hello, World!114514");

        let ro_throwable: LocalObject<JavaThrowable> = env.typed_catch().unwrap();
        assert!(env.is_same_object(Some(&*ro_throwable), Some(&*o_exception)));
    })
}
