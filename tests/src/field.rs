use typed_jni::{
    LocalClass, LocalObject, TypedCallExt, TypedClassExt, TypedFieldAccessExt, TypedStringExt, builtin::JavaString,
    define_java_class,
};

use crate::{compile_file_and_load_classes, with_java_vm};

// Modify the test_get_set_static_field function to add write tests for all fields
#[test]
fn test_get_set_static_field() {
    with_java_vm(|env| {
        define_java_class!(JavaTest, "Test");

        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "Test",
            r#"public class Test {
                public static boolean staticBoolean = true;
                public static byte staticByte = 127;
                public static char staticChar = 'A';
                public static short staticShort = 32767;
                public static int staticInt = 2147483647;
                public static long staticLong = 9223372036854775807L;
                public static float staticFloat = (float) java.lang.Math.PI;
                public static double staticDouble = (double) java.lang.Math.E;
                public static String staticString = "Hello, World";
            }"#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();

        // Test getting static fields
        let boolean_val: bool = env.typed_get_field(&c_test, "staticBoolean").unwrap();
        assert!(boolean_val);

        let byte_val: i8 = env.typed_get_field(&c_test, "staticByte").unwrap();
        assert_eq!(byte_val, 127);

        let char_val: u16 = env.typed_get_field(&c_test, "staticChar").unwrap();
        assert_eq!(char_val, 'A' as u16);

        let short_val: i16 = env.typed_get_field(&c_test, "staticShort").unwrap();
        assert_eq!(short_val, 32767);

        let int_val: i32 = env.typed_get_field(&c_test, "staticInt").unwrap();
        assert_eq!(int_val, 2147483647);

        let long_val: i64 = env.typed_get_field(&c_test, "staticLong").unwrap();
        assert_eq!(long_val, 9223372036854775807);

        let float_val: f32 = env.typed_get_field(&c_test, "staticFloat").unwrap();
        assert!((float_val - std::f32::consts::PI).abs() < 0.0001);

        let double_val: f64 = env.typed_get_field(&c_test, "staticDouble").unwrap();
        assert!((double_val - std::f64::consts::E).abs() < 0.00001);

        let string_val: LocalObject<JavaString> = env.typed_get_field(&c_test, "staticString").unwrap();
        assert_eq!(env.typed_get_string(&string_val), "Hello, World");

        // Test setting static fields - add write tests for all types
        env.typed_set_field(&c_test, "staticBoolean", false).unwrap();
        let new_boolean_val: bool = env.typed_get_field(&c_test, "staticBoolean").unwrap();
        assert!(!new_boolean_val);

        env.typed_set_field(&c_test, "staticByte", -128i8).unwrap();
        let new_byte_val: i8 = env.typed_get_field(&c_test, "staticByte").unwrap();
        assert_eq!(new_byte_val, -128);

        env.typed_set_field(&c_test, "staticChar", 'Z' as u16).unwrap();
        let new_char_val: u16 = env.typed_get_field(&c_test, "staticChar").unwrap();
        assert_eq!(new_char_val, 'Z' as u16);

        env.typed_set_field(&c_test, "staticShort", -32768i16).unwrap();
        let new_short_val: i16 = env.typed_get_field(&c_test, "staticShort").unwrap();
        assert_eq!(new_short_val, -32768);

        env.typed_set_field(&c_test, "staticInt", -42).unwrap();
        let new_int_val: i32 = env.typed_get_field(&c_test, "staticInt").unwrap();
        assert_eq!(new_int_val, -42);

        env.typed_set_field(&c_test, "staticLong", -9223372036854775808i64).unwrap();
        let new_long_val: i64 = env.typed_get_field(&c_test, "staticLong").unwrap();
        assert_eq!(new_long_val, -9223372036854775808);

        env.typed_set_field(&c_test, "staticFloat", std::f32::consts::E).unwrap();
        let new_float_val: f32 = env.typed_get_field(&c_test, "staticFloat").unwrap();
        assert!((new_float_val - std::f32::consts::E).abs() < 0.0001);

        env.typed_set_field(&c_test, "staticDouble", std::f64::consts::PI).unwrap();
        let new_double_val: f64 = env.typed_get_field(&c_test, "staticDouble").unwrap();
        assert!((new_double_val - std::f64::consts::PI).abs() < 0.00001);

        let new_string = env.typed_new_string("Updated String");
        env.typed_set_field(&c_test, "staticString", &new_string).unwrap();
        let new_string_val: LocalObject<JavaString> = env.typed_get_field(&c_test, "staticString").unwrap();
        assert_eq!(env.typed_get_string(&new_string_val), "Updated String");
    })
}

#[test]
fn test_get_set_instance_field() {
    with_java_vm(|env| {
        define_java_class!(JavaTest, "Test");

        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "Test",
            r#"public class Test {
                public boolean instanceBoolean = true;
                public byte instanceByte = 127;
                public char instanceChar = 'A';
                public short instanceShort = 32767;
                public int instanceInt = 2147483647;
                public long instanceLong = 9223372036854775807L;
                public float instanceFloat = (float) java.lang.Math.PI;
                public double instanceDouble = (double) java.lang.Math.E;
                public String instanceString = "Hello, World";
                
                public Test() {}
            }"#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();
        let o_test: LocalObject<JavaTest> = env.typed_new_object(&c_test, ()).unwrap();

        // Test getting instance fields
        let boolean_val: bool = env.typed_get_field(&o_test, "instanceBoolean").unwrap();
        assert!(boolean_val);

        let byte_val: i8 = env.typed_get_field(&o_test, "instanceByte").unwrap();
        assert_eq!(byte_val, 127);

        let char_val: u16 = env.typed_get_field(&o_test, "instanceChar").unwrap();
        assert_eq!(char_val, 'A' as u16);

        let short_val: i16 = env.typed_get_field(&o_test, "instanceShort").unwrap();
        assert_eq!(short_val, 32767);

        let int_val: i32 = env.typed_get_field(&o_test, "instanceInt").unwrap();
        assert_eq!(int_val, 2147483647);

        let long_val: i64 = env.typed_get_field(&o_test, "instanceLong").unwrap();
        assert_eq!(long_val, 9223372036854775807);

        let float_val: f32 = env.typed_get_field(&o_test, "instanceFloat").unwrap();
        assert!((float_val - std::f32::consts::PI).abs() < 0.0001);

        let double_val: f64 = env.typed_get_field(&o_test, "instanceDouble").unwrap();
        assert!((double_val - std::f64::consts::E).abs() < 0.00001);

        let string_val: LocalObject<JavaString> = env.typed_get_field(&o_test, "instanceString").unwrap();
        assert_eq!(env.typed_get_string(&string_val), "Hello, World");

        // Test setting instance fields - add write tests for all types
        env.typed_set_field(&o_test, "instanceBoolean", false).unwrap();
        let new_boolean_val: bool = env.typed_get_field(&o_test, "instanceBoolean").unwrap();
        assert!(!new_boolean_val);

        env.typed_set_field(&o_test, "instanceByte", -128i8).unwrap();
        let new_byte_val: i8 = env.typed_get_field(&o_test, "instanceByte").unwrap();
        assert_eq!(new_byte_val, -128);

        env.typed_set_field(&o_test, "instanceChar", 'Z' as u16).unwrap();
        let new_char_val: u16 = env.typed_get_field(&o_test, "instanceChar").unwrap();
        assert_eq!(new_char_val, 'Z' as u16);

        env.typed_set_field(&o_test, "instanceShort", -32768i16).unwrap();
        let new_short_val: i16 = env.typed_get_field(&o_test, "instanceShort").unwrap();
        assert_eq!(new_short_val, -32768);

        env.typed_set_field(&o_test, "instanceInt", -42).unwrap();
        let new_int_val: i32 = env.typed_get_field(&o_test, "instanceInt").unwrap();
        assert_eq!(new_int_val, -42);

        env.typed_set_field(&o_test, "instanceLong", -9223372036854775808i64).unwrap();
        let new_long_val: i64 = env.typed_get_field(&o_test, "instanceLong").unwrap();
        assert_eq!(new_long_val, -9223372036854775808);

        env.typed_set_field(&o_test, "instanceFloat", std::f32::consts::E).unwrap();
        let new_float_val: f32 = env.typed_get_field(&o_test, "instanceFloat").unwrap();
        assert!((new_float_val - std::f32::consts::E).abs() < 0.0001);

        env.typed_set_field(&o_test, "instanceDouble", std::f64::consts::PI).unwrap();
        let new_double_val: f64 = env.typed_get_field(&o_test, "instanceDouble").unwrap();
        assert!((new_double_val - std::f64::consts::PI).abs() < 0.00001);

        let new_string = env.typed_new_string("Updated Instance String");
        env.typed_set_field(&o_test, "instanceString", &new_string).unwrap();
        let new_string_val: LocalObject<JavaString> = env.typed_get_field(&o_test, "instanceString").unwrap();
        assert_eq!(env.typed_get_string(&new_string_val), "Updated Instance String");
    })
}

#[test]
fn test_field_access_with_inner_class() {
    with_java_vm(|env| {
        define_java_class!(JavaTest, "Test");
        define_java_class!(JavaInner, "Test$Inner");

        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "Test",
            r#"public class Test {
                public Inner innerClass = new Inner();
                
                public static class Inner {
                    public int value = 42;
                    public String name = "InnerClass";
                }
            }"#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();
        let o_test: LocalObject<JavaTest> = env.typed_new_object(&c_test, ()).unwrap();

        let inner_obj: LocalObject<JavaInner> = env.typed_get_field(&o_test, "innerClass").unwrap();
        let value: i32 = env.typed_get_field(&inner_obj, "value").unwrap();
        let name: LocalObject<JavaString> = env.typed_get_field(&inner_obj, "name").unwrap();

        assert_eq!(value, 42);
        assert_eq!(env.typed_get_string(&name), "InnerClass");

        env.typed_set_field(&inner_obj, "value", 100).unwrap();
        let new_value: i32 = env.typed_get_field(&inner_obj, "value").unwrap();
        assert_eq!(new_value, 100);

        let new_name = env.typed_new_string("NewInnerClass");
        env.typed_set_field(&inner_obj, "name", &new_name).unwrap();
        let new_name_val: LocalObject<JavaString> = env.typed_get_field(&inner_obj, "name").unwrap();
        assert_eq!(env.typed_get_string(&new_name_val), "NewInnerClass");
    })
}

#[test]
fn test_nullable_field_handling() {
    with_java_vm(|env| {
        define_java_class!(JavaTest, "Test");

        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "Test",
            r#"public class Test {
                public String nullableString = null;
                public String nonNullableString = "NonNull";
                
                public Test() {}
            }"#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();
        let o_test: LocalObject<JavaTest> = env.typed_new_object(&c_test, ()).unwrap();

        // Test: Get nullable field
        let nullable_result: Option<LocalObject<JavaString>> = env.typed_get_field(&o_test, "nullableString").unwrap();
        assert!(nullable_result.is_none());

        // Test: Get non-null field
        let non_null_result: Option<LocalObject<JavaString>> = env.typed_get_field(&o_test, "nonNullableString").unwrap();
        assert!(non_null_result.is_some());
        assert_eq!(env.typed_get_string(&non_null_result.unwrap()), "NonNull");

        // Test: Set nullable field to null
        env.typed_set_field(&o_test, "nullableString", typed_jni::Null::<JavaString>::NULL)
            .unwrap();
        let still_null_result: Option<LocalObject<JavaString>> = env.typed_get_field(&o_test, "nullableString").unwrap();
        assert!(still_null_result.is_none());

        // Test: Set non-null value to nullable field
        let new_string = env.typed_new_string("NowNotNull");
        env.typed_set_field(&o_test, "nullableString", &new_string).unwrap();
        let now_not_null_result: Option<LocalObject<JavaString>> = env.typed_get_field(&o_test, "nullableString").unwrap();
        assert!(now_not_null_result.is_some());
        assert_eq!(env.typed_get_string(&now_not_null_result.unwrap()), "NowNotNull");
    })
}
