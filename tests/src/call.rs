use typed_jni::{
    DynArg, LocalClass, LocalObject, Null, TypedCallExt, TypedClassExt, TypedObjectExt, TypedStringExt, builtin::JavaString,
    define_java_class,
};

use crate::{compile_file_and_load_classes, with_java_vm};

#[test]
fn test_basic_call_static_method() {
    with_java_vm(|env| {
        define_java_class!(JavaTest, "Test");

        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "Test",
            r#"public class Test {
                public static void callVoid() {
                }

                public static boolean callBoolean(boolean v) {
                    return v;
                }

                public static byte callByte(byte v) {
                    return v;
                }

                public static char callChar(char v) {
                    return v;
                }

                public static short callShort(short v) {
                    return v;
                }

                public static int callInt(int v) {
                    return v;
                }

                public static long callLong(long v) {
                    return v;
                }

                public static float callFloat(float v) {
                    return v;
                }

                public static double callDouble(double v) {
                    return v;
                }

                public static String callString(String v) {
                    return v;
                }
            }"#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();

        // Test void type
        let _: () = env.typed_call_method(&c_test, "callVoid", ()).unwrap();

        // Test boolean type
        let boolean_ret: bool = env.typed_call_method(&c_test, "callBoolean", (true,)).unwrap();
        assert!(boolean_ret);

        let boolean_ret: bool = env.typed_call_method(&c_test, "callBoolean", (false,)).unwrap();
        assert!(!boolean_ret);

        // Test byte type
        let byte_ret: i8 = env.typed_call_method(&c_test, "callByte", (127i8,)).unwrap();
        assert_eq!(byte_ret, 127);

        let byte_ret: i8 = env.typed_call_method(&c_test, "callByte", (-128i8,)).unwrap();
        assert_eq!(byte_ret, -128);

        // Test char type
        let char_ret: u16 = env.typed_call_method(&c_test, "callChar", ('A' as u16,)).unwrap();
        assert_eq!(char_ret, 'A' as u16);

        let char_ret: u16 = env.typed_call_method(&c_test, "callChar", (0x0041u16,)).unwrap();
        assert_eq!(char_ret, 0x0041);

        // Test short type
        let short_ret: i16 = env.typed_call_method(&c_test, "callShort", (32767i16,)).unwrap();
        assert_eq!(short_ret, 32767);

        let short_ret: i16 = env.typed_call_method(&c_test, "callShort", (-32768i16,)).unwrap();
        assert_eq!(short_ret, -32768);

        // Test int type
        let int_ret: i32 = env.typed_call_method(&c_test, "callInt", (2147483647i32,)).unwrap();
        assert_eq!(int_ret, 2147483647);

        let int_ret: i32 = env.typed_call_method(&c_test, "callInt", (-2147483648i32,)).unwrap();
        assert_eq!(int_ret, -2147483648);

        // Test long type
        let long_ret: i64 = env.typed_call_method(&c_test, "callLong", (9223372036854775807i64,)).unwrap();
        assert_eq!(long_ret, 9223372036854775807);

        let long_ret: i64 = env
            .typed_call_method(&c_test, "callLong", (-9223372036854775808i64,))
            .unwrap();
        assert_eq!(long_ret, -9223372036854775808i64);

        // Test float type
        let float_ret: f32 = env.typed_call_method(&c_test, "callFloat", (std::f32::consts::PI,)).unwrap();
        assert!((float_ret - std::f32::consts::PI).abs() < 0.0001);

        // Test double type
        let double_ret: f64 = env.typed_call_method(&c_test, "callDouble", (std::f64::consts::E,)).unwrap();
        assert!((double_ret - std::f64::consts::E).abs() < 0.00001);

        // Test String type
        let string_ret: LocalObject<JavaString> = env
            .typed_call_method(&c_test, "callString", (env.typed_new_string("Hello World"),))
            .unwrap();
        assert_eq!(env.typed_get_string(&string_ret), "Hello World");
    })
}

#[test]
fn test_basic_call_method() {
    with_java_vm(|env| {
        define_java_class!(JavaTest, "Test");

        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "Test",
            r#"public class Test {
                public void call() {
                }

                public boolean call(boolean v) {
                    return v;
                }

                public byte call(byte v) {
                    return v;
                }

                public char call(char v) {
                    return v;
                }

                public short call(short v) {
                    return v;
                }

                public int call(int v) {
                    return v;
                }

                public long call(long v) {
                    return v;
                }

                public float call(float v) {
                    return v;
                }

                public double call(double v) {
                    return v;
                }

                public String call(String v) {
                    return v;
                }
            }"#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();
        let o_test: LocalObject<JavaTest> = env.typed_new_object(&c_test, ()).unwrap();

        // Test void type
        let _: () = env.typed_call_method(&o_test, "call", ()).unwrap();

        // Test boolean type
        let boolean_ret: bool = env.typed_call_method(&o_test, "call", (true,)).unwrap();
        assert!(boolean_ret);

        let boolean_ret: bool = env.typed_call_method(&o_test, "call", (false,)).unwrap();
        assert!(!boolean_ret);

        // Test byte type
        let byte_ret: i8 = env.typed_call_method(&o_test, "call", (127i8,)).unwrap();
        assert_eq!(byte_ret, 127);

        let byte_ret: i8 = env.typed_call_method(&o_test, "call", (-128i8,)).unwrap();
        assert_eq!(byte_ret, -128);

        // Test char type
        let char_ret: u16 = env.typed_call_method(&o_test, "call", ('A' as u16,)).unwrap();
        assert_eq!(char_ret, 'A' as u16);

        let char_ret: u16 = env.typed_call_method(&o_test, "call", (0x0041u16,)).unwrap();
        assert_eq!(char_ret, 0x0041);

        // Test short type
        let short_ret: i16 = env.typed_call_method(&o_test, "call", (32767i16,)).unwrap();
        assert_eq!(short_ret, 32767);

        let short_ret: i16 = env.typed_call_method(&o_test, "call", (-32768i16,)).unwrap();
        assert_eq!(short_ret, -32768);

        // Test int type
        let int_ret: i32 = env.typed_call_method(&o_test, "call", (2147483647i32,)).unwrap();
        assert_eq!(int_ret, 2147483647);

        let int_ret: i32 = env.typed_call_method(&o_test, "call", (-2147483648i32,)).unwrap();
        assert_eq!(int_ret, -2147483648);

        // Test long type
        let long_ret: i64 = env.typed_call_method(&o_test, "call", (9223372036854775807i64,)).unwrap();
        assert_eq!(long_ret, 9223372036854775807);

        let long_ret: i64 = env.typed_call_method(&o_test, "call", (-9223372036854775808i64,)).unwrap();
        assert_eq!(long_ret, -9223372036854775808i64);

        // Test float type
        let float_ret: f32 = env.typed_call_method(&o_test, "call", (std::f32::consts::PI,)).unwrap();
        assert!((float_ret - std::f32::consts::PI).abs() < 0.0001);

        // Test double type
        let double_ret: f64 = env.typed_call_method(&o_test, "call", (std::f64::consts::E,)).unwrap();
        assert!((double_ret - std::f64::consts::E).abs() < 0.00001);

        // Test String type
        let string_ret: LocalObject<JavaString> = env
            .typed_call_method(&o_test, "call", (env.typed_new_string("Hello World"),))
            .unwrap();
        assert_eq!(env.typed_get_string(&string_ret), "Hello World");
    })
}

#[test]
fn test_call_overloaded_instance_method() {
    with_java_vm(|env| {
        define_java_class!(JavaTest, "Test");

        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "Test",
            r#"public class Test {
                // Overloaded instance method: same name, different parameter types
                public int add(int a, int b) {
                    return a + b;
                }

                public double add(double a, double b) {
                    return a + b;
                }

                public String add(String a, String b) {
                    return a + b;
                }

                // Overloaded instance method: same name, different parameter counts
                public int add(int a) {
                    return a + 10;
                }

                // No-arg constructor
                public Test() {
                }
            }"#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();

        // Create an instance of the Test class
        let instance: LocalObject<JavaTest> = env.typed_new_object(&c_test, ()).unwrap();

        // Test overloaded method 1: two int parameters
        let int_result: i32 = env.typed_call_method(&instance, "add", (10i32, 20i32)).unwrap();
        assert_eq!(int_result, 30);

        // Test overloaded method 2: two double parameters
        let double_result: f64 = env.typed_call_method(&instance, "add", (10.5f64, 20.5f64)).unwrap();
        assert!((double_result - 31.0f64).abs() < 0.0001);

        // Test overloaded method 3: two String parameters
        let s1 = env.typed_new_string("Hello ");
        let s2 = env.typed_new_string("World");
        let string_result: LocalObject<JavaString> = env.typed_call_method(&instance, "add", (&s1, &s2)).unwrap();
        assert_eq!(env.typed_get_string(&string_result), "Hello World");

        // Test overloaded method 4: one int parameter
        let single_int_result: i32 = env.typed_call_method(&instance, "add", (5i32,)).unwrap();
        assert_eq!(single_int_result, 15);
    })
}

#[test]
fn test_nullable_object_handling() {
    with_java_vm(|env| {
        define_java_class!(JavaTest, "Test");

        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "Test",
            r#"public class Test {
                // Test receiving nullable object parameters and returning objects
                public String processNullableString(String input) {
                    // If input is null, return "NULL_VALUE"
                    if (input == null) {
                        return "NULL_VALUE";
                    }
                    // Otherwise return the uppercase form of the input string
                    return input.toUpperCase();
                }

                // Test method that returns null
                public String returnNullString(boolean shouldReturnNull) {
                    if (shouldReturnNull) {
                        return null;
                    }
                    return "NOT_NULL";
                }

                // Test passing null to a method that accepts two parameters
                public String concatStrings(String a, String b) {
                    if (a == null && b == null) {
                        return "BOTH_NULL";
                    } else if (a == null) {
                        return "A_NULL:" + b;
                    } else if (b == null) {
                        return a + ":B_NULL";
                    }
                    return a + b;
                }

                // No-arg constructor
                public Test() {
                }
            }"#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();
        let instance: LocalObject<JavaTest> = env.typed_new_object(&c_test, ()).unwrap();

        // Test 1: Pass non-null parameter, receive non-null return value
        let non_null_str = env.typed_new_string("hello");
        let result1: LocalObject<JavaString> = env
            .typed_call_method(&instance, "processNullableString", (&non_null_str,))
            .unwrap();
        assert_eq!(env.typed_get_string(&result1), "HELLO");

        // Test 2: Pass null parameter, receive non-null return value
        let result2: LocalObject<JavaString> = env
            .typed_call_method(&instance, "processNullableString", (Null::<JavaString>::NULL,))
            .unwrap();
        assert_eq!(env.typed_get_string(&result2), "NULL_VALUE");

        // Test 3: Receive null return value
        let result3: Option<LocalObject<JavaString>> = env.typed_call_method(&instance, "returnNullString", (true,)).unwrap();
        assert!(result3.is_none());

        // Test 4: Receive non-null return value
        let result4: Option<LocalObject<JavaString>> = env.typed_call_method(&instance, "returnNullString", (false,)).unwrap();
        assert!(result4.is_some());
        assert_eq!(env.typed_get_string(&result4.unwrap()), "NOT_NULL");

        // Test 5: Pass two non-null parameters
        let str_a = env.typed_new_string("prefix_");
        let str_b = env.typed_new_string("suffix");
        let result5: LocalObject<JavaString> = env.typed_call_method(&instance, "concatStrings", (&str_a, &str_b)).unwrap();
        assert_eq!(env.typed_get_string(&result5), "prefix_suffix");

        // Test 6: Pass first parameter as null, second as non-null
        let result6: LocalObject<JavaString> = env
            .typed_call_method(&instance, "concatStrings", (Null::<JavaString>::NULL, &str_b))
            .unwrap();
        assert_eq!(env.typed_get_string(&result6), "A_NULL:suffix");

        // Test 7: Pass first parameter as non-null, second as null
        let result7: LocalObject<JavaString> = env
            .typed_call_method(&instance, "concatStrings", (&str_a, Null::<JavaString>::NULL))
            .unwrap();
        assert_eq!(env.typed_get_string(&result7), "prefix_:B_NULL");

        // Test 8: Pass two null parameters
        let result8: LocalObject<JavaString> = env
            .typed_call_method(
                &instance,
                "concatStrings",
                (Null::<JavaString>::NULL, Null::<JavaString>::NULL),
            )
            .unwrap();
        assert_eq!(env.typed_get_string(&result8), "BOTH_NULL");
    })
}

#[test]
fn test_dyn_args() {
    with_java_vm(|env| {
        define_java_class!(JavaTest, "Test");

        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "Test",
            r#"public class Test {
                public String concat(String a1, int a2, long a3, byte a4) {
                    return String.format("%s_%d_%d_%d", a1, a2, a3, a4);
                }
            }"#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();
        let o_test: LocalObject<JavaTest> = env.typed_new_object(&c_test, ()).unwrap();

        let str = env.typed_new_string("hello");
        let result1: LocalObject<JavaString> = env
            .typed_call_method(&o_test, "concat", &[&str as &dyn DynArg, &123i32, &456i64, &7i8])
            .unwrap();
        let result2: LocalObject<JavaString> = env
            .typed_call_method(&o_test, "concat", &[&str as &dyn DynArg, &123i32, &456i64, &7i8][..]) // as slice
            .unwrap();
        assert_eq!(env.typed_get_string(&result1), "hello_123_456_7");
        assert_eq!(env.typed_get_string(&result2), "hello_123_456_7");
    })
}

#[test]
fn test_exception_handling() {
    with_java_vm(|env| {
        define_java_class!(JavaTest, "Test");

        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "Test",
            r#"public class Test {
                // Test static method throwing exception
                public static void throwNullPointerException() {
                    String str = null;
                    str.length(); // Will throw NullPointerException
                }
                
                // Test static method throwing exception with message
                public static int throwIllegalArgumentException(String message) {
                    throw new IllegalArgumentException(message);
                }
                
                // Test instance method throwing exception
                public void throwRuntimeException() {
                    throw new RuntimeException("Testing runtime exception");
                }
                
                // Test instance method throwing specific type of exception
                public String throwExceptionWithType(String exceptionType) {
                    if ("ArithmeticException".equals(exceptionType)) {
                        int result = 1 / 0; // Will throw ArithmeticException
                    } else if ("ArrayIndexOutOfBoundsException".equals(exceptionType)) {
                        int[] arr = new int[5];
                        return String.valueOf(arr[10]); // Will throw ArrayIndexOutOfBoundsException
                    }
                    return "Normal return value";
                }
                
                // No-arg constructor
                public Test() {
                }
            }"#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();
        let instance: LocalObject<JavaTest> = env.typed_new_object(&c_test, ()).unwrap();

        // Test 1: Call static method that throws NullPointerException
        let result1 = env.typed_call_method::<(), _, _>(&c_test, "throwNullPointerException", ());
        assert!(result1.is_err());
        let exception1 = result1.unwrap_err();
        let exception_class_name = env.typed_to_string(&env.typed_get_object_class(&exception1)).unwrap();
        assert!(exception_class_name.contains("NullPointerException"));

        // Test 2: Call static method that throws IllegalArgumentException with message
        let error_message = "Testing parameter exception";
        let result2 = env.typed_call_method::<i32, _, _>(
            &c_test,
            "throwIllegalArgumentException",
            (env.typed_new_string(error_message),),
        );
        assert!(result2.is_err());
        let exception2 = result2.unwrap_err();
        let exception_class_name = env.typed_to_string(&env.typed_get_object_class(&exception2)).unwrap();
        assert!(exception_class_name.contains("IllegalArgumentException"));

        // Test 3: Call instance method that throws RuntimeException
        let result3 = env.typed_call_method::<(), _, _>(&instance, "throwRuntimeException", ());
        assert!(result3.is_err());
        let exception3 = result3.unwrap_err();
        let exception_class_name = env.typed_to_string(&env.typed_get_object_class(&exception3)).unwrap();
        assert!(exception_class_name.contains("RuntimeException"));

        // Test 4: Call instance method that throws different types of exceptions
        let result4 = env.typed_call_method::<LocalObject<JavaString>, _, _>(
            &instance,
            "throwExceptionWithType",
            (env.typed_new_string("ArithmeticException"),),
        );
        assert!(result4.is_err());
        let exception4 = result4.err().unwrap();
        let exception_class_name = env.typed_to_string(&env.typed_get_object_class(&exception4)).unwrap();
        assert!(exception_class_name.contains("ArithmeticException"));

        // Test 5: Call instance method that throws ArrayIndexOutOfBoundsException
        let result5 = env.typed_call_method::<LocalObject<JavaString>, _, _>(
            &instance,
            "throwExceptionWithType",
            (env.typed_new_string("ArrayIndexOutOfBoundsException"),),
        );
        assert!(result5.is_err());
        let exception5 = result5.err().unwrap();
        let exception_class_name = env.typed_to_string(&env.typed_get_object_class(&exception5)).unwrap();
        assert!(exception_class_name.contains("ArrayIndexOutOfBoundsException"));

        // Test 6: Call instance method with unknown exception type
        let result6 = env.typed_call_method::<LocalObject<JavaString>, _, _>(
            &instance,
            "throwExceptionWithType",
            (env.typed_new_string("UnknownType"),),
        );
        assert!(result6.is_ok());
        assert_eq!(env.typed_get_string(&result6.unwrap()), "Normal return value");
    })
}

#[test]
fn test_object_creation() {
    with_java_vm(|env| {
        define_java_class!(JavaTest, "Test");

        let (_dir, loader) = compile_file_and_load_classes(
            env,
            "Test",
            r#"public class Test {
                private int intValue;
                private String stringValue;
                private boolean booleanValue;
                
                // No-arg constructor
                public Test() {
                    this.intValue = 0;
                    this.stringValue = "Default";
                    this.booleanValue = false;
                }
                
                // Constructor with one int parameter
                public Test(int value) {
                    this.intValue = value;
                    this.stringValue = "IntegerConstructor";
                    this.booleanValue = true;
                }
                
                // Constructor with String parameter
                public Test(String value) {
                    this.intValue = -1;
                    this.stringValue = value;
                    this.booleanValue = false;
                }
                
                // Constructor with multiple parameters of different types
                public Test(int intVal, String strVal, boolean boolVal) {
                    this.intValue = intVal;
                    this.stringValue = strVal;
                    this.booleanValue = boolVal;
                }
                
                // Constructor that throws exception
                public Test(boolean shouldThrow) {
                    if (shouldThrow) {
                        throw new IllegalArgumentException("Testing constructor exception");
                    }
                    this.intValue = 42;
                    this.stringValue = "SafeConstructor";
                    this.booleanValue = false;
                }
                
                // Methods to get properties
                public int getIntValue() {
                    return intValue;
                }
                
                public String getStringValue() {
                    return stringValue;
                }
                
                public boolean isBooleanValue() {
                    return booleanValue;
                }
            }"#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();

        // Test 1: Create object with no-arg constructor
        let instance1: LocalObject<JavaTest> = env.typed_new_object(&c_test, ()).unwrap();
        assert_eq!(env.typed_call_method::<i32, _, _>(&instance1, "getIntValue", ()).unwrap(), 0);
        assert_eq!(
            env.typed_get_string(&env.typed_call_method(&instance1, "getStringValue", ()).unwrap()),
            "Default"
        );
        assert_eq!(
            env.typed_call_method::<bool, _, _>(&instance1, "isBooleanValue", ()).unwrap(),
            false
        );

        // Test 2: Call constructor with int parameter to create object
        let instance2: LocalObject<JavaTest> = env.typed_new_object(&c_test, (42i32,)).unwrap();
        assert_eq!(env.typed_call_method::<i32, _, _>(&instance2, "getIntValue", ()).unwrap(), 42);
        assert_eq!(
            env.typed_get_string(&env.typed_call_method(&instance2, "getStringValue", ()).unwrap()),
            "IntegerConstructor"
        );
        assert_eq!(
            env.typed_call_method::<bool, _, _>(&instance2, "isBooleanValue", ()).unwrap(),
            true
        );

        // Test 3: Call constructor with String parameter to create object
        let test_str = env.typed_new_string("CustomString");
        let instance3: LocalObject<JavaTest> = env.typed_new_object(&c_test, (&test_str,)).unwrap();
        assert_eq!(env.typed_call_method::<i32, _, _>(&instance3, "getIntValue", ()).unwrap(), -1);
        assert_eq!(
            env.typed_get_string(&env.typed_call_method(&instance3, "getStringValue", ()).unwrap()),
            "CustomString"
        );
        assert_eq!(
            env.typed_call_method::<bool, _, _>(&instance3, "isBooleanValue", ()).unwrap(),
            false
        );

        // Test 4: Call constructor with multiple parameters of different types to create object
        let multi_param_str = env.typed_new_string("MultiParam");
        let instance4: LocalObject<JavaTest> = env.typed_new_object(&c_test, (100i32, &multi_param_str, true)).unwrap();
        assert_eq!(
            env.typed_call_method::<i32, _, _>(&instance4, "getIntValue", ()).unwrap(),
            100
        );
        assert_eq!(
            env.typed_get_string(&env.typed_call_method(&instance4, "getStringValue", ()).unwrap()),
            "MultiParam"
        );
        assert_eq!(
            env.typed_call_method::<bool, _, _>(&instance4, "isBooleanValue", ()).unwrap(),
            true
        );

        // Test 5: Call constructor with boolean parameter to create object
        let instance5: LocalObject<JavaTest> = env.typed_new_object(&c_test, (false,)).unwrap();
        assert_eq!(env.typed_call_method::<i32, _, _>(&instance5, "getIntValue", ()).unwrap(), 42);
        assert_eq!(
            env.typed_get_string(&env.typed_call_method(&instance5, "getStringValue", ()).unwrap()),
            "SafeConstructor"
        );
        assert_eq!(
            env.typed_call_method::<bool, _, _>(&instance5, "isBooleanValue", ()).unwrap(),
            false
        );

        // Test 6: Call constructor with boolean parameter that throws exception
        let result6 = env.typed_new_object::<JavaTest, _, _>(&c_test, (true,));
        assert!(result6.is_err());
        let exception6 = result6.err().unwrap();
        let exception_class_name = env.typed_to_string(&env.typed_get_object_class(&exception6)).unwrap();
        assert!(exception_class_name.contains("IllegalArgumentException"));
    })
}
