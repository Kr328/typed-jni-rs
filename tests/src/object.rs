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

        // Test getting the class of an object
        let o_test_class = env.typed_get_object_class(&o_test);
        let o_test_subclass_class = env.typed_get_object_class(&o_test_subclass);

        // Verify that the obtained class is correct
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

        // Test if an object is an instance of a class
        assert!(env.typed_is_instance_of(&o_test, &c_test));
        assert!(env.typed_is_instance_of(&o_test_subclass, &c_test_subclass));
        assert!(env.typed_is_instance_of(&o_test_subclass, &c_test)); // Subclass instance is also an instance of the parent class
        assert!(!env.typed_is_instance_of(&o_test, &c_test_subclass)); // Parent class instance is not an instance of the subclass
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

        // Test if a class can be assigned from another class
        assert!(env.typed_is_assignable_from(&c_test, &c_test)); // A class can be assigned from itself
        assert!(env.typed_is_assignable_from(&c_test_subclass, &c_test_subclass)); // A class can be assigned from itself
        assert!(env.typed_is_assignable_from(&c_test_subclass, &c_test)); // Parent class can be assigned from subclass
        assert!(!env.typed_is_assignable_from(&c_test, &c_test_subclass)); // Subclass cannot be assigned from parent class
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

        // Test casting a subclass object to a parent class type (should succeed)
        let cast_result = env.typed_cast(&o_test_subclass, &c_test);
        assert!(cast_result.is_ok());

        // Test casting a parent class object to a subclass type (should fail)
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

        // Test the toString method of an object
        let string_result = env.typed_to_string(&o_test);
        assert!(string_result.is_ok());
        assert_eq!(string_result.unwrap(), "CustomTestObject");
    })
}

#[test]
fn test_typed_to_string_null() {
    with_java_vm(|env| {
        define_java_class!(JavaTest, "Test");

        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "Test",
            r#"public class Test {
                @Override
                public String toString() {
                    return null;
                }
            }"#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();
        let o_test: LocalObject<JavaTest> = env.typed_new_object(&c_test, ()).unwrap();

        // Test the toString method of an object
        let string_result = env.typed_to_string(&o_test);
        assert!(string_result.is_err());
        assert!(
            env.typed_to_string(&string_result.err().unwrap())
                .unwrap()
                .contains("NullPointerException")
        );
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

        // Test the hashCode method of a custom object
        let hash_code_result = env.typed_hash_code(&o_test);
        assert!(hash_code_result.is_ok());
        assert_eq!(hash_code_result.unwrap(), 12345);

        // Test the hashCode method of a String object
        let o_string: LocalObject<JavaString> = env.typed_new_string("test-string");
        let string_hash_code_result = env.typed_hash_code(&o_string);
        assert!(string_hash_code_result.is_ok());
        // The hashCode of a String is deterministic, but the specific value may vary by Java version, so we only verify the call succeeds
    })
}