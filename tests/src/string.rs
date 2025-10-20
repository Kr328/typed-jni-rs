use typed_jni::{LocalObject, TypedStringExt, builtin::JavaString};

use crate::with_java_vm;

#[test]
fn test_convert_string() {
    with_java_vm(|env| {
        let content = include_str!("../testdata/unicode-test.html");

        let o_string: LocalObject<JavaString> = env.typed_new_string(&content);
        let r_content: String = env.typed_get_string(&o_string);

        assert_eq!(content, r_content);
    })
}
