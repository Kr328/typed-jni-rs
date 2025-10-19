use typed_jni::{
    Array, LocalClass, LocalObject, TypedCallExt, TypedClassExt, TypedRefExt,
    builtin::{JavaClassLoader, JavaString},
    define_java_class,
};

use crate::{compile_file_and_load_classes, with_java_vm};

#[test]
fn test_find_array_class() {
    with_java_vm(|env| {
        env.typed_find_class::<Array<bool>>().unwrap();
        env.typed_find_class::<Array<JavaString>>().unwrap();
        env.typed_find_class::<Array<Array<bool>>>().unwrap();
        env.typed_find_class::<Array<Array<JavaString>>>().unwrap();

        let c_cl: LocalClass<JavaClassLoader> = env.typed_find_class().unwrap();
        let o_system_cl: LocalObject<JavaClassLoader> = env.typed_call_method(&c_cl, "getSystemClassLoader", ()).unwrap();

        env.typed_find_class_in_class_loader::<Array<bool>, _>(&o_system_cl).unwrap();
        env.typed_find_class_in_class_loader::<Array<JavaString>, _>(&o_system_cl)
            .unwrap();
        env.typed_find_class_in_class_loader::<Array<Array<bool>>, _>(&o_system_cl)
            .unwrap();
        env.typed_find_class_in_class_loader::<Array<Array<JavaString>>, _>(&o_system_cl)
            .unwrap();
    })
}

#[test]
fn test_get_class_loader() {
    with_java_vm(|env| {
        // 查找一个内置类，例如 String 类
        let string_class: LocalClass<JavaString> = env.typed_find_class().unwrap();

        // 获取 String 类的类加载器（应该是 null，因为它是引导类加载器加载的）
        let class_loader = env
            .typed_get_class_loader(&env.typed_new_local_ref(&string_class).into_class_object())
            .unwrap();

        // 验证获取到的类加载器为 None（表示 null）
        assert!(
            class_loader.is_none(),
            "String class should be loaded by bootstrap class loader"
        );

        // 测试一个自定义类的类加载器
        // 编译一个简单的测试类并使用自定义类加载器加载
        let (_temp, custom_loader) = compile_file_and_load_classes(env, "TestClass", r#"public class TestClass {}"#);

        // 定义测试类类型
        define_java_class!(JavaTestClass, "TestClass");

        // 使用自定义类加载器查找测试类
        let test_class: LocalClass<JavaTestClass> = env.typed_find_class_in_class_loader(&custom_loader).unwrap();

        // 获取测试类的类加载器
        let test_class_loader = env.typed_get_class_loader(&test_class.into_class_object()).unwrap();

        // 验证获取到的类加载器
        assert!(test_class_loader.is_some(), "TestClass should have a class loader");
        assert!(env.is_same_object(test_class_loader.as_deref(), Some(&*custom_loader)))
    })
}
