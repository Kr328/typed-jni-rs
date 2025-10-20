use typed_jni::{
    Array, LocalClass, LocalObject, TypedArrayExt, TypedCallExt, TypedClassExt, TypedObjectArrayExt, TypedPrimitiveArrayExt,
    TypedStringExt, builtin::JavaString,
};

use crate::with_java_vm;

#[test]
fn test_string_array() {
    with_java_vm(|env| {
        let length = 5;
        let array = (0..length).map(|idx| format!("string: {idx}")).collect::<Vec<_>>();

        let o_array: LocalObject<Array<JavaString>> = env
            .typed_new_array(&env.typed_find_class().unwrap(), array.len() as _)
            .unwrap();
        for (index, s) in array.iter().enumerate() {
            env.typed_set_array_element(&o_array, index as _, Some(&env.typed_new_string(s)))
                .unwrap();
        }

        let r_length = env.typed_get_array_length(&o_array).unwrap();
        let mut r_array = Vec::with_capacity(r_length as _);
        for index in 0..r_length {
            let s: Option<LocalObject<JavaString>> = env.typed_get_array_element(&o_array, index).unwrap();

            r_array.push(env.typed_get_string(&s.unwrap()));
        }

        assert_eq!(array, r_array);
    })
}

#[test]
fn test_bool_array() {
    with_java_vm(|env| {
        let length = 7;
        let array: Vec<bool> = (0..length).map(|idx| idx % 3 == 0).collect();

        let o_array: LocalObject<Array<bool>> = env.typed_new_primitive_array(array.len() as _).unwrap();
        env.typed_set_array_region(&o_array, 0, &array).unwrap();

        let mut r_array = vec![false; array.len()];
        env.typed_get_array_region(&o_array, 0, &mut r_array).unwrap();

        assert_eq!(array, r_array);
    })
}

#[test]
fn test_int_array_access() {
    with_java_vm(|env| {
        let array = env.typed_new_primitive_array::<i32>(8).unwrap();

        let mut elements = env.typed_get_array_elements(&array).unwrap();

        elements[0] = 1;
        elements[1] = 2;
        elements[2] = 3;
        elements[3] = 4;

        elements.commit();

        let mut buf = [0i32; 8];

        env.typed_get_array_region(&array, 0, &mut buf[..]).unwrap();

        assert_eq!(buf, [1, 2, 3, 4, 0, 0, 0, 0]);

        let mut elements = env.typed_get_array_elements(&array).unwrap();

        elements[4] = 1;
        elements[5] = 2;
        elements[6] = 3;
        elements[7] = 4;

        drop(elements);

        let mut buf = [0i32; 8];

        env.typed_get_array_region(&array, 0, &mut buf).unwrap();

        assert_eq!(buf, [1, 2, 3, 4, 0, 0, 0, 0]);

        env.typed_set_array_region(&array, 4, &[8, 9, 10, 11]).unwrap();

        let buf = env.typed_get_array_elements(&array).unwrap();

        assert_eq!(*buf, [1, 2, 3, 4, 8, 9, 10, 11])
    })
}

#[test]
fn test_bytes_access() {
    let s = "Hello你好こんにちは안녕하세요";

    with_java_vm(|env| {
        let array: LocalObject<Array<i8>> = env.typed_new_primitive_array::<i8>(s.as_bytes().len() as _).unwrap();

        let mut elements = env.typed_get_bytes_array_elements(&array).unwrap();

        elements.copy_from_slice(s.as_bytes());

        elements.commit();

        let java_s: LocalClass<JavaString> = env.typed_find_class::<JavaString>().unwrap();
        let java_s: LocalObject<JavaString> = env.typed_new_object(&java_s, (&array,)).unwrap();
        assert_eq!(env.typed_get_string(&java_s), s);

        let array: LocalObject<Array<i8>> = env.typed_call_method(&java_s, "getBytes", ()).unwrap();
        assert_eq!(&*env.typed_get_bytes_array_elements(&array).unwrap(), s.as_bytes());
    })
}
