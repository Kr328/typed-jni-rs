use typed_jni::{
    LocalClass, LocalObject, TypedCallExt, TypedClassExt, TypedObjectExt, TypedStringExt, builtin::JavaString, define_java_class,
};

use crate::{compile_file_and_load_classes, with_java_vm};

#[test]
fn test_typed_get_object_class() {
    with_java_vm(|env| {
        define_java_class!(JavaTest, "Test");
        define_java_class!(JavaTestSubclass, "Test$TestSubclass");

        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "Test",
            r#"public class Test {
                public static class TestSubclass extends Test {}
                }
            "#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();
        let c_test_subclass: LocalClass<JavaTestSubclass> = env.typed_find_class_in_class_loader(&loader).unwrap();

        let o_test: LocalObject<JavaTest> = env.typed_new_object(&c_test, ()).unwrap();
        let o_test_subclass: LocalObject<JavaTestSubclass> = env.typed_new_object(&c_test_subclass, ()).unwrap();

        // 测试获取对象的类
        let o_test_class = env.typed_get_object_class(&o_test);
        let o_test_subclass_class = env.typed_get_object_class(&o_test_subclass);

        // 验证获取的类是正确的
        assert!(env.is_same_object(Some(&*o_test_class), Some(&*c_test)));
        assert!(env.is_same_object(Some(&*o_test_subclass_class), Some(&*c_test_subclass)));
    })
}

#[test]
fn test_typed_is_instance_of() {
    with_java_vm(|env| {
        define_java_class!(JavaTest, "Test");
        define_java_class!(JavaTestSubclass, "Test$TestSubclass");

        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "Test",
            r#"public class Test {
                public static class TestSubclass extends Test {}
            }
            "#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();
        let c_test_subclass: LocalClass<JavaTestSubclass> = env.typed_find_class_in_class_loader(&loader).unwrap();

        let o_test: LocalObject<JavaTest> = env.typed_new_object(&c_test, ()).unwrap();
        let o_test_subclass: LocalObject<JavaTestSubclass> = env.typed_new_object(&c_test_subclass, ()).unwrap();

        // 测试对象是否是类的实例
        assert!(env.typed_is_instance_of(&o_test, &c_test));
        assert!(env.typed_is_instance_of(&o_test_subclass, &c_test_subclass));
        assert!(env.typed_is_instance_of(&o_test_subclass, &c_test)); // 子类实例也是父类的实例
        assert!(!env.typed_is_instance_of(&o_test, &c_test_subclass)); // 父类实例不是子类的实例
    })
}

#[test]
fn test_typed_is_assignable_from() {
    with_java_vm(|env| {
        define_java_class!(JavaTest, "Test");
        define_java_class!(JavaTestSubclass, "Test$TestSubclass");

        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "Test",
            r#"public class Test {
                public static class TestSubclass extends Test {}
            }
            "#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();
        let c_test_subclass: LocalClass<JavaTestSubclass> = env.typed_find_class_in_class_loader(&loader).unwrap();

        // 测试类是否可以从另一个类赋值
        assert!(env.typed_is_assignable_from(&c_test, &c_test)); // 类可以从自身赋值
        assert!(env.typed_is_assignable_from(&c_test_subclass, &c_test_subclass)); // 类可以从自身赋值
        assert!(env.typed_is_assignable_from(&c_test_subclass, &c_test)); // 父类可以从子类赋值
        assert!(!env.typed_is_assignable_from(&c_test, &c_test_subclass)); // 子类不能从父类赋值
    })
}

#[test]
fn test_typed_cast() {
    with_java_vm(|env| {
        define_java_class!(JavaTest, "Test");
        define_java_class!(JavaTestSubclass, "Test$TestSubclass");

        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "Test",
            r#"public class Test {
                public static class TestSubclass extends Test {}
            }
            "#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();
        let c_test_subclass: LocalClass<JavaTestSubclass> = env.typed_find_class_in_class_loader(&loader).unwrap();

        let o_test: LocalObject<JavaTest> = env.typed_new_object(&c_test, ()).unwrap();
        let o_test_subclass: LocalObject<JavaTestSubclass> = env.typed_new_object(&c_test_subclass, ()).unwrap();

        // 测试将子类对象转换为父类类型（应该成功）
        let cast_result = env.typed_cast(&o_test_subclass, &c_test);
        assert!(cast_result.is_ok());

        // 测试将父类对象转换为子类类型（应该失败）
        let cast_result = env.typed_cast(&o_test, &c_test_subclass);
        assert!(cast_result.is_err());
    })
}

#[test]
fn test_typed_to_string() {
    with_java_vm(|env| {
        define_java_class!(JavaTest, "Test");

        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "Test",
            r#"public class Test {
                @Override
                public String toString() {
                    return "CustomTestObject";
                }
            }"#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();
        let o_test: LocalObject<JavaTest> = env.typed_new_object(&c_test, ()).unwrap();

        // 测试对象的toString方法
        let string_result = env.typed_to_string(&o_test);
        assert!(string_result.is_ok());
        assert_eq!(string_result.unwrap(), "CustomTestObject");
    })
}

#[test]
fn test_typed_hash_code() {
    with_java_vm(|env| {
        define_java_class!(JavaTest, "Test");

        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "Test",
            r#"public class Test {
                @Override
                public int hashCode() {
                    return 12345;
                }
            }"#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();
        let o_test: LocalObject<JavaTest> = env.typed_new_object(&c_test, ()).unwrap();

        // 测试自定义对象的hashCode方法
        let hash_code_result = env.typed_hash_code(&o_test);
        assert!(hash_code_result.is_ok());
        assert_eq!(hash_code_result.unwrap(), 12345);

        // 测试String对象的hashCode方法
        let o_string: LocalObject<JavaString> = env.typed_new_string("test-string");
        let string_hash_code_result = env.typed_hash_code(&o_string);
        assert!(string_hash_code_result.is_ok());
        // String的hashCode是确定性的，但具体值可能因Java版本而异，这里只验证调用成功
    })
}
