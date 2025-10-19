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

        // 测试 void 类型
        let _: () = env.typed_call_method(&c_test, "callVoid", ()).unwrap();

        // 测试 boolean 类型
        let boolean_ret: bool = env.typed_call_method(&c_test, "callBoolean", (true,)).unwrap();
        assert!(boolean_ret);

        let boolean_ret: bool = env.typed_call_method(&c_test, "callBoolean", (false,)).unwrap();
        assert!(!boolean_ret);

        // 测试 byte 类型
        let byte_ret: i8 = env.typed_call_method(&c_test, "callByte", (127i8,)).unwrap();
        assert_eq!(byte_ret, 127);

        let byte_ret: i8 = env.typed_call_method(&c_test, "callByte", (-128i8,)).unwrap();
        assert_eq!(byte_ret, -128);

        // 测试 char 类型
        let char_ret: u16 = env.typed_call_method(&c_test, "callChar", ('A' as u16,)).unwrap();
        assert_eq!(char_ret, 'A' as u16);

        let char_ret: u16 = env.typed_call_method(&c_test, "callChar", (0x0041u16,)).unwrap();
        assert_eq!(char_ret, 0x0041);

        // 测试 short 类型
        let short_ret: i16 = env.typed_call_method(&c_test, "callShort", (32767i16,)).unwrap();
        assert_eq!(short_ret, 32767);

        let short_ret: i16 = env.typed_call_method(&c_test, "callShort", (-32768i16,)).unwrap();
        assert_eq!(short_ret, -32768);

        // 测试 int 类型
        let int_ret: i32 = env.typed_call_method(&c_test, "callInt", (2147483647i32,)).unwrap();
        assert_eq!(int_ret, 2147483647);

        let int_ret: i32 = env.typed_call_method(&c_test, "callInt", (-2147483648i32,)).unwrap();
        assert_eq!(int_ret, -2147483648);

        // 测试 long 类型
        let long_ret: i64 = env.typed_call_method(&c_test, "callLong", (9223372036854775807i64,)).unwrap();
        assert_eq!(long_ret, 9223372036854775807);

        let long_ret: i64 = env
            .typed_call_method(&c_test, "callLong", (-9223372036854775808i64,))
            .unwrap();
        assert_eq!(long_ret, -9223372036854775808i64);

        // 测试 float 类型
        let float_ret: f32 = env.typed_call_method(&c_test, "callFloat", (std::f32::consts::PI,)).unwrap();
        assert!((float_ret - std::f32::consts::PI).abs() < 0.0001);

        // 测试 double 类型
        let double_ret: f64 = env.typed_call_method(&c_test, "callDouble", (std::f64::consts::E,)).unwrap();
        assert!((double_ret - std::f64::consts::E).abs() < 0.00001);

        // 测试 String 类型
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

        // 测试 void 类型
        let _: () = env.typed_call_method(&o_test, "call", ()).unwrap();

        // 测试 boolean 类型
        let boolean_ret: bool = env.typed_call_method(&o_test, "call", (true,)).unwrap();
        assert!(boolean_ret);

        let boolean_ret: bool = env.typed_call_method(&o_test, "call", (false,)).unwrap();
        assert!(!boolean_ret);

        // 测试 byte 类型
        let byte_ret: i8 = env.typed_call_method(&o_test, "call", (127i8,)).unwrap();
        assert_eq!(byte_ret, 127);

        let byte_ret: i8 = env.typed_call_method(&o_test, "call", (-128i8,)).unwrap();
        assert_eq!(byte_ret, -128);

        // 测试 char 类型
        let char_ret: u16 = env.typed_call_method(&o_test, "call", ('A' as u16,)).unwrap();
        assert_eq!(char_ret, 'A' as u16);

        let char_ret: u16 = env.typed_call_method(&o_test, "call", (0x0041u16,)).unwrap();
        assert_eq!(char_ret, 0x0041);

        // 测试 short 类型
        let short_ret: i16 = env.typed_call_method(&o_test, "call", (32767i16,)).unwrap();
        assert_eq!(short_ret, 32767);

        let short_ret: i16 = env.typed_call_method(&o_test, "call", (-32768i16,)).unwrap();
        assert_eq!(short_ret, -32768);

        // 测试 int 类型
        let int_ret: i32 = env.typed_call_method(&o_test, "call", (2147483647i32,)).unwrap();
        assert_eq!(int_ret, 2147483647);

        let int_ret: i32 = env.typed_call_method(&o_test, "call", (-2147483648i32,)).unwrap();
        assert_eq!(int_ret, -2147483648);

        // 测试 long 类型
        let long_ret: i64 = env.typed_call_method(&o_test, "call", (9223372036854775807i64,)).unwrap();
        assert_eq!(long_ret, 9223372036854775807);

        let long_ret: i64 = env.typed_call_method(&o_test, "call", (-9223372036854775808i64,)).unwrap();
        assert_eq!(long_ret, -9223372036854775808i64);

        // 测试 float 类型
        let float_ret: f32 = env.typed_call_method(&o_test, "call", (std::f32::consts::PI,)).unwrap();
        assert!((float_ret - std::f32::consts::PI).abs() < 0.0001);

        // 测试 double 类型
        let double_ret: f64 = env.typed_call_method(&o_test, "call", (std::f64::consts::E,)).unwrap();
        assert!((double_ret - std::f64::consts::E).abs() < 0.00001);

        // 测试 String 类型
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
                // 重载的实例方法：相同名称，不同参数类型
                public int add(int a, int b) {
                    return a + b;
                }

                public double add(double a, double b) {
                    return a + b;
                }

                public String add(String a, String b) {
                    return a + b;
                }

                // 重载的实例方法：相同名称，不同参数数量
                public int add(int a) {
                    return a + 10;
                }

                // 无参构造函数
                public Test() {
                }
            }"#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();

        // 创建Test类的实例
        let instance: LocalObject<JavaTest> = env.typed_new_object(&c_test, ()).unwrap();

        // 测试重载方法1：两个int参数
        let int_result: i32 = env.typed_call_method(&instance, "add", (10i32, 20i32)).unwrap();
        assert_eq!(int_result, 30);

        // 测试重载方法2：两个double参数
        let double_result: f64 = env.typed_call_method(&instance, "add", (10.5f64, 20.5f64)).unwrap();
        assert!((double_result - 31.0f64).abs() < 0.0001);

        // 测试重载方法3：两个String参数
        let s1 = env.typed_new_string("Hello ");
        let s2 = env.typed_new_string("World");
        let string_result: LocalObject<JavaString> = env.typed_call_method(&instance, "add", (&s1, &s2)).unwrap();
        assert_eq!(env.typed_get_string(&string_result), "Hello World");

        // 测试重载方法4：一个int参数
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
                // 测试接收可空对象参数并返回对象
                public String processNullableString(String input) {
                    // 如果输入为null，返回"NULL_VALUE"
                    if (input == null) {
                        return "NULL_VALUE";
                    }
                    // 否则返回输入字符串的大写形式
                    return input.toUpperCase();
                }

                // 测试返回null的方法
                public String returnNullString(boolean shouldReturnNull) {
                    if (shouldReturnNull) {
                        return null;
                    }
                    return "NOT_NULL";
                }

                // 测试传递null给接受两个参数的方法
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

                // 无参构造函数
                public Test() {
                }
            }"#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();
        let instance: LocalObject<JavaTest> = env.typed_new_object(&c_test, ()).unwrap();

        // 测试1：传递非null参数，接收非null返回值
        let non_null_str = env.typed_new_string("hello");
        let result1: LocalObject<JavaString> = env
            .typed_call_method(&instance, "processNullableString", (&non_null_str,))
            .unwrap();
        assert_eq!(env.typed_get_string(&result1), "HELLO");

        // 测试2：传递null参数，接收非null返回值
        let result2: LocalObject<JavaString> = env
            .typed_call_method(&instance, "processNullableString", (Null::<JavaString>::NULL,))
            .unwrap();
        assert_eq!(env.typed_get_string(&result2), "NULL_VALUE");

        // 测试3：接收null返回值
        let result3: Option<LocalObject<JavaString>> = env.typed_call_method(&instance, "returnNullString", (true,)).unwrap();
        assert!(result3.is_none());

        // 测试4：接收非null返回值
        let result4: Option<LocalObject<JavaString>> = env.typed_call_method(&instance, "returnNullString", (false,)).unwrap();
        assert!(result4.is_some());
        assert_eq!(env.typed_get_string(&result4.unwrap()), "NOT_NULL");

        // 测试5：传递两个非null参数
        let str_a = env.typed_new_string("prefix_");
        let str_b = env.typed_new_string("suffix");
        let result5: LocalObject<JavaString> = env.typed_call_method(&instance, "concatStrings", (&str_a, &str_b)).unwrap();
        assert_eq!(env.typed_get_string(&result5), "prefix_suffix");

        // 测试6：传递第一个参数为null，第二个为非null
        let result6: LocalObject<JavaString> = env
            .typed_call_method(&instance, "concatStrings", (Null::<JavaString>::NULL, &str_b))
            .unwrap();
        assert_eq!(env.typed_get_string(&result6), "A_NULL:suffix");

        // 测试7：传递第一个参数为非null，第二个为null
        let result7: LocalObject<JavaString> = env
            .typed_call_method(&instance, "concatStrings", (&str_a, Null::<JavaString>::NULL))
            .unwrap();
        assert_eq!(env.typed_get_string(&result7), "prefix_:B_NULL");

        // 测试8：传递两个null参数
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
                // 测试静态方法抛出异常
                public static void throwNullPointerException() {
                    String str = null;
                    str.length(); // 会抛出NullPointerException
                }
                
                // 测试静态方法抛出带消息的异常
                public static int throwIllegalArgumentException(String message) {
                    throw new IllegalArgumentException(message);
                }
                
                // 测试实例方法抛出异常
                public void throwRuntimeException() {
                    throw new RuntimeException("测试运行时异常");
                }
                
                // 测试实例方法抛出特定类型的异常
                public String throwExceptionWithType(String exceptionType) {
                    if ("ArithmeticException".equals(exceptionType)) {
                        int result = 1 / 0; // 会抛出ArithmeticException
                    } else if ("ArrayIndexOutOfBoundsException".equals(exceptionType)) {
                        int[] arr = new int[5];
                        return String.valueOf(arr[10]); // 会抛出ArrayIndexOutOfBoundsException
                    }
                    return "正常返回值";
                }
                
                // 无参构造函数
                public Test() {
                }
            }"#,
        );

        let c_test: LocalClass<JavaTest> = env.typed_find_class_in_class_loader(&loader).unwrap();
        let instance: LocalObject<JavaTest> = env.typed_new_object(&c_test, ()).unwrap();

        // 测试1：调用会抛出NullPointerException的静态方法
        let result1 = env.typed_call_method::<(), _, _>(&c_test, "throwNullPointerException", ());
        assert!(result1.is_err());
        let exception1 = result1.unwrap_err();
        let exception_class_name = env.typed_to_string(&env.typed_get_object_class(&exception1)).unwrap();
        assert!(exception_class_name.contains("NullPointerException"));

        // 测试2：调用会抛出带消息的IllegalArgumentException的静态方法
        let error_message = "测试参数异常";
        let result2 = env.typed_call_method::<i32, _, _>(
            &c_test,
            "throwIllegalArgumentException",
            (env.typed_new_string(error_message),),
        );
        assert!(result2.is_err());
        let exception2 = result2.unwrap_err();
        let exception_class_name = env.typed_to_string(&env.typed_get_object_class(&exception2)).unwrap();
        assert!(exception_class_name.contains("IllegalArgumentException"));

        // 测试3：调用会抛出RuntimeException的实例方法
        let result3 = env.typed_call_method::<(), _, _>(&instance, "throwRuntimeException", ());
        assert!(result3.is_err());
        let exception3 = result3.unwrap_err();
        let exception_class_name = env.typed_to_string(&env.typed_get_object_class(&exception3)).unwrap();
        assert!(exception_class_name.contains("RuntimeException"));

        // 测试4：调用会抛出不同类型异常的实例方法
        let result4 = env.typed_call_method::<LocalObject<JavaString>, _, _>(
            &instance,
            "throwExceptionWithType",
            (env.typed_new_string("ArithmeticException"),),
        );
        assert!(result4.is_err());
        let exception4 = result4.err().unwrap();
        let exception_class_name = env.typed_to_string(&env.typed_get_object_class(&exception4)).unwrap();
        assert!(exception_class_name.contains("ArithmeticException"));

        let result5 = env.typed_call_method::<LocalObject<JavaString>, _, _>(
            &instance,
            "throwExceptionWithType",
            (env.typed_new_string("ArrayIndexOutOfBoundsException"),),
        );
        assert!(result5.is_err());
        let exception5 = result5.err().unwrap();
        let exception_class_name = env.typed_to_string(&env.typed_get_object_class(&exception5)).unwrap();
        assert!(exception_class_name.contains("ArrayIndexOutOfBoundsException"));

        // 测试6：正常情况不会抛出异常
        let result6 = env.typed_call_method(&instance, "throwExceptionWithType", (env.typed_new_string("UnknownType"),));
        assert!(result6.is_ok());
        assert_eq!(env.typed_get_string(&result6.unwrap()), "正常返回值");
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
                
                // 无参构造函数
                public Test() {
                    this.intValue = 0;
                    this.stringValue = "Default";
                    this.booleanValue = false;
                }
                
                // 带一个int参数的构造函数
                public Test(int value) {
                    this.intValue = value;
                    this.stringValue = "IntegerConstructor";
                    this.booleanValue = true;
                }
                
                // 带String参数的构造函数
                public Test(String value) {
                    this.intValue = -1;
                    this.stringValue = value;
                    this.booleanValue = false;
                }
                
                // 带多个不同类型参数的构造函数
                public Test(int intVal, String strVal, boolean boolVal) {
                    this.intValue = intVal;
                    this.stringValue = strVal;
                    this.booleanValue = boolVal;
                }
                
                // 会抛出异常的构造函数
                public Test(boolean shouldThrow) {
                    if (shouldThrow) {
                        throw new IllegalArgumentException("测试构造函数异常");
                    }
                    this.intValue = 42;
                    this.stringValue = "SafeConstructor";
                    this.booleanValue = false;
                }
                
                // 获取属性的方法
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

        // 测试1: 无参构造函数创建对象
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

        // 测试2: 带int参数的构造函数创建对象
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

        // 测试3: 带String参数的构造函数创建对象
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

        // 测试4: 带多个不同类型参数的构造函数创建对象
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

        // 测试5: 正常的boolean参数构造函数
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

        // 测试6: 构造函数抛出异常
        let result6 = env.typed_new_object::<JavaTest, _, _>(&c_test, (true,));
        assert!(result6.is_err());
        let exception6 = result6.err().unwrap();
        let exception_class_name = env.typed_to_string(&env.typed_get_object_class(&exception6)).unwrap();
        assert!(exception_class_name.contains("IllegalArgumentException"));
    })
}
