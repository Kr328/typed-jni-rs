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
        // Find a built-in class, such as String class
        let string_class: LocalClass<JavaString> = env.typed_find_class().unwrap();

        // Get the class loader of String class (should be null as it's loaded by bootstrap class loader)
        let class_loader = env
            .typed_get_class_loader(&env.typed_new_local_ref(&string_class).into_class_object())
            .unwrap();

        // Verify that the obtained class loader is None (indicating null)
        assert!(
            class_loader.is_none(),
            "String class should be loaded by bootstrap class loader"
        );

        // Test the class loader of a custom class
        // Compile a simple test class and load it using a custom class loader
        let (_temp, custom_loader) = compile_file_and_load_classes(env, "TestClass", r#"public class TestClass {}"#);

        // Define test class type
        define_java_class!(JavaTestClass, "TestClass");

        // Find test class using custom class loader
        let test_class: LocalClass<JavaTestClass> = env.typed_find_class_in_class_loader(&custom_loader).unwrap();

        // Get the class loader of the test class
        let test_class_loader = env.typed_get_class_loader(&test_class.into_class_object()).unwrap();

        // Verify the obtained class loader
        assert!(test_class_loader.is_some(), "TestClass should have a class loader");
        assert!(env.is_same_object(test_class_loader.as_deref(), Some(&*custom_loader)))
    })
}
