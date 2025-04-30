#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(unused_qualifications)]
#![allow(dead_code)]

pub const JNI_FALSE: bool = false;
pub const JNI_TRUE: bool = true;
pub const JNI_OK: i32 = 0;
pub const JNI_ERR: i32 = -1;
pub const JNI_EDETACHED: i32 = -2;
pub const JNI_EVERSION: i32 = -3;
pub const JNI_ENOMEM: i32 = -4;
pub const JNI_EEXIST: i32 = -5;
pub const JNI_EINVAL: i32 = -6;
pub const JNI_COMMIT: u32 = 1;
pub const JNI_ABORT: u32 = 2;
pub const JNI_VERSION_1_1: u32 = 65537;
pub const JNI_VERSION_1_2: u32 = 65538;
pub const JNI_VERSION_1_4: u32 = 65540;
pub const JNI_VERSION_1_6: u32 = 65542;
pub const JNI_VERSION_1_8: u32 = 65544;
pub const JNI_VERSION_9: u32 = 589824;
pub const JNI_VERSION_10: u32 = 655360;
pub const JNI_VERSION_19: u32 = 1245184;
pub const JNI_VERSION_20: u32 = 1310720;
pub const JNI_VERSION_21: u32 = 1376256;

pub type jint = i32;
pub type jlong = i64;
pub type jbyte = i8;
pub type jboolean = bool;
pub type jchar = u16;
pub type jshort = i16;
pub type jfloat = f32;
pub type jdouble = f64;
pub type jsize = jint;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _jobject;
pub type jobject = *mut _jobject;
pub type jclass = jobject;
pub type jthrowable = jobject;
pub type jstring = jobject;
pub type jarray = jobject;
pub type jbooleanArray = jarray;
pub type jbyteArray = jarray;
pub type jcharArray = jarray;
pub type jshortArray = jarray;
pub type jintArray = jarray;
pub type jlongArray = jarray;
pub type jfloatArray = jarray;
pub type jdoubleArray = jarray;
pub type jobjectArray = jarray;
pub type jweak = jobject;
#[repr(C)]
#[derive(Copy, Clone)]
pub union jvalue {
    pub z: jboolean,
    pub b: jbyte,
    pub c: jchar,
    pub s: jshort,
    pub i: jint,
    pub j: jlong,
    pub f: jfloat,
    pub d: jdouble,
    pub l: jobject,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _jfieldID;
pub type jfieldID = *mut _jfieldID;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _jmethodID;
pub type jmethodID = *mut _jmethodID;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum jobjectRefType {
    JNIInvalidRefType,
    JNILocalRefType,
    JNIGlobalRefType,
    JNIWeakGlobalRefType,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct JNINativeMethod {
    pub name: *mut ::core::ffi::c_char,
    pub signature: *mut ::core::ffi::c_char,
    pub fnPtr: *mut ::core::ffi::c_void,
}
pub type JNIEnv = *const JNINativeInterface_;
pub type JavaVM = *const JNIInvokeInterface_;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct JNINativeInterface_ {
    pub reserved0: *mut ::core::ffi::c_void,
    pub reserved1: *mut ::core::ffi::c_void,
    pub reserved2: *mut ::core::ffi::c_void,
    pub reserved3: *mut ::core::ffi::c_void,
    pub GetVersion: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv) -> jint>,
    pub DefineClass: ::core::option::Option<
        unsafe extern "C" fn(
            env: *mut JNIEnv,
            name: *const ::core::ffi::c_char,
            loader: jobject,
            buf: *const jbyte,
            len: jsize,
        ) -> jclass,
    >,
    pub FindClass: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, name: *const ::core::ffi::c_char) -> jclass>,
    pub FromReflectedMethod: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, method: jobject) -> jmethodID>,
    pub FromReflectedField: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, field: jobject) -> jfieldID>,
    pub ToReflectedMethod: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, cls: jclass, methodID: jmethodID, isStatic: jboolean) -> jobject,
    >,
    pub GetSuperclass: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, sub: jclass) -> jclass>,
    pub IsAssignableFrom: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, sub: jclass, sup: jclass) -> jboolean>,
    pub ToReflectedField: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, cls: jclass, fieldID: jfieldID, isStatic: jboolean) -> jobject,
    >,
    pub Throw: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jthrowable) -> jint>,
    pub ThrowNew:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, msg: *const ::core::ffi::c_char) -> jint>,
    pub ExceptionOccurred: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv) -> jthrowable>,
    pub ExceptionDescribe: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv)>,
    pub ExceptionClear: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv)>,
    pub FatalError: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, msg: *const ::core::ffi::c_char)>,
    pub PushLocalFrame: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, capacity: jint) -> jint>,
    pub PopLocalFrame: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, result: jobject) -> jobject>,
    pub NewGlobalRef: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, lobj: jobject) -> jobject>,
    pub DeleteGlobalRef: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, gref: jobject)>,
    pub DeleteLocalRef: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject)>,
    pub IsSameObject: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj1: jobject, obj2: jobject) -> jboolean>,
    pub NewLocalRef: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, ref_: jobject) -> jobject>,
    pub EnsureLocalCapacity: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, capacity: jint) -> jint>,
    pub AllocObject: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass) -> jobject>,
    pub NewObject:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jobject>,
    pub NewObjectV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *mut ()) -> jobject,
    >,
    pub NewObjectA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *const jvalue) -> jobject,
    >,
    pub GetObjectClass: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject) -> jclass>,
    pub IsInstanceOf: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass) -> jboolean>,
    pub GetMethodID: ::core::option::Option<
        unsafe extern "C" fn(
            env: *mut JNIEnv,
            clazz: jclass,
            name: *const ::core::ffi::c_char,
            sig: *const ::core::ffi::c_char,
        ) -> jmethodID,
    >,
    pub CallObjectMethod:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) -> jobject>,
    pub CallObjectMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: *mut ()) -> jobject,
    >,
    pub CallObjectMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: *const jvalue) -> jobject,
    >,
    pub CallBooleanMethod:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) -> jboolean>,
    pub CallBooleanMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: *mut ()) -> jboolean,
    >,
    pub CallBooleanMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: *const jvalue) -> jboolean,
    >,
    pub CallByteMethod:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) -> jbyte>,
    pub CallByteMethodV:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: *mut ()) -> jbyte>,
    pub CallByteMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: *const jvalue) -> jbyte,
    >,
    pub CallCharMethod:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) -> jchar>,
    pub CallCharMethodV:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: *mut ()) -> jchar>,
    pub CallCharMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: *const jvalue) -> jchar,
    >,
    pub CallShortMethod:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) -> jshort>,
    pub CallShortMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: *mut ()) -> jshort,
    >,
    pub CallShortMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: *const jvalue) -> jshort,
    >,
    pub CallIntMethod:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) -> jint>,
    pub CallIntMethodV:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: *mut ()) -> jint>,
    pub CallIntMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: *const jvalue) -> jint,
    >,
    pub CallLongMethod:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) -> jlong>,
    pub CallLongMethodV:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: *mut ()) -> jlong>,
    pub CallLongMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: *const jvalue) -> jlong,
    >,
    pub CallFloatMethod:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) -> jfloat>,
    pub CallFloatMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: *mut ()) -> jfloat,
    >,
    pub CallFloatMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: *const jvalue) -> jfloat,
    >,
    pub CallDoubleMethod:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...) -> jdouble>,
    pub CallDoubleMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: *mut ()) -> jdouble,
    >,
    pub CallDoubleMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: *const jvalue) -> jdouble,
    >,
    pub CallVoidMethod: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, ...)>,
    pub CallVoidMethodV:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: *mut ())>,
    pub CallVoidMethodA:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, methodID: jmethodID, args: *const jvalue)>,
    pub CallNonvirtualObjectMethod: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, ...) -> jobject,
    >,
    pub CallNonvirtualObjectMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, args: *mut ()) -> jobject,
    >,
    pub CallNonvirtualObjectMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, args: *const jvalue) -> jobject,
    >,
    pub CallNonvirtualBooleanMethod: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, ...) -> jboolean,
    >,
    pub CallNonvirtualBooleanMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, args: *mut ()) -> jboolean,
    >,
    pub CallNonvirtualBooleanMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, args: *const jvalue) -> jboolean,
    >,
    pub CallNonvirtualByteMethod: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, ...) -> jbyte,
    >,
    pub CallNonvirtualByteMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, args: *mut ()) -> jbyte,
    >,
    pub CallNonvirtualByteMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, args: *const jvalue) -> jbyte,
    >,
    pub CallNonvirtualCharMethod: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, ...) -> jchar,
    >,
    pub CallNonvirtualCharMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, args: *mut ()) -> jchar,
    >,
    pub CallNonvirtualCharMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, args: *const jvalue) -> jchar,
    >,
    pub CallNonvirtualShortMethod: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, ...) -> jshort,
    >,
    pub CallNonvirtualShortMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, args: *mut ()) -> jshort,
    >,
    pub CallNonvirtualShortMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, args: *const jvalue) -> jshort,
    >,
    pub CallNonvirtualIntMethod: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, ...) -> jint,
    >,
    pub CallNonvirtualIntMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, args: *mut ()) -> jint,
    >,
    pub CallNonvirtualIntMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, args: *const jvalue) -> jint,
    >,
    pub CallNonvirtualLongMethod: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, ...) -> jlong,
    >,
    pub CallNonvirtualLongMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, args: *mut ()) -> jlong,
    >,
    pub CallNonvirtualLongMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, args: *const jvalue) -> jlong,
    >,
    pub CallNonvirtualFloatMethod: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, ...) -> jfloat,
    >,
    pub CallNonvirtualFloatMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, args: *mut ()) -> jfloat,
    >,
    pub CallNonvirtualFloatMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, args: *const jvalue) -> jfloat,
    >,
    pub CallNonvirtualDoubleMethod: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, ...) -> jdouble,
    >,
    pub CallNonvirtualDoubleMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, args: *mut ()) -> jdouble,
    >,
    pub CallNonvirtualDoubleMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, args: *const jvalue) -> jdouble,
    >,
    pub CallNonvirtualVoidMethod:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, ...)>,
    pub CallNonvirtualVoidMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, args: *mut ()),
    >,
    pub CallNonvirtualVoidMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, clazz: jclass, methodID: jmethodID, args: *const jvalue),
    >,
    pub GetFieldID: ::core::option::Option<
        unsafe extern "C" fn(
            env: *mut JNIEnv,
            clazz: jclass,
            name: *const ::core::ffi::c_char,
            sig: *const ::core::ffi::c_char,
        ) -> jfieldID,
    >,
    pub GetObjectField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID) -> jobject>,
    pub GetBooleanField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID) -> jboolean>,
    pub GetByteField: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID) -> jbyte>,
    pub GetCharField: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID) -> jchar>,
    pub GetShortField: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID) -> jshort>,
    pub GetIntField: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID) -> jint>,
    pub GetLongField: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID) -> jlong>,
    pub GetFloatField: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID) -> jfloat>,
    pub GetDoubleField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID) -> jdouble>,
    pub SetObjectField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID, val: jobject)>,
    pub SetBooleanField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID, val: jboolean)>,
    pub SetByteField: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID, val: jbyte)>,
    pub SetCharField: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID, val: jchar)>,
    pub SetShortField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID, val: jshort)>,
    pub SetIntField: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID, val: jint)>,
    pub SetLongField: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID, val: jlong)>,
    pub SetFloatField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID, val: jfloat)>,
    pub SetDoubleField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject, fieldID: jfieldID, val: jdouble)>,
    pub GetStaticMethodID: ::core::option::Option<
        unsafe extern "C" fn(
            env: *mut JNIEnv,
            clazz: jclass,
            name: *const ::core::ffi::c_char,
            sig: *const ::core::ffi::c_char,
        ) -> jmethodID,
    >,
    pub CallStaticObjectMethod:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jobject>,
    pub CallStaticObjectMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *mut ()) -> jobject,
    >,
    pub CallStaticObjectMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *const jvalue) -> jobject,
    >,
    pub CallStaticBooleanMethod:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jboolean>,
    pub CallStaticBooleanMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *mut ()) -> jboolean,
    >,
    pub CallStaticBooleanMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *const jvalue) -> jboolean,
    >,
    pub CallStaticByteMethod:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jbyte>,
    pub CallStaticByteMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *mut ()) -> jbyte,
    >,
    pub CallStaticByteMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *const jvalue) -> jbyte,
    >,
    pub CallStaticCharMethod:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jchar>,
    pub CallStaticCharMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *mut ()) -> jchar,
    >,
    pub CallStaticCharMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *const jvalue) -> jchar,
    >,
    pub CallStaticShortMethod:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jshort>,
    pub CallStaticShortMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *mut ()) -> jshort,
    >,
    pub CallStaticShortMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *const jvalue) -> jshort,
    >,
    pub CallStaticIntMethod:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jint>,
    pub CallStaticIntMethodV:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *mut ()) -> jint>,
    pub CallStaticIntMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *const jvalue) -> jint,
    >,
    pub CallStaticLongMethod:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jlong>,
    pub CallStaticLongMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *mut ()) -> jlong,
    >,
    pub CallStaticLongMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *const jvalue) -> jlong,
    >,
    pub CallStaticFloatMethod:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jfloat>,
    pub CallStaticFloatMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *mut ()) -> jfloat,
    >,
    pub CallStaticFloatMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *const jvalue) -> jfloat,
    >,
    pub CallStaticDoubleMethod:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, ...) -> jdouble>,
    pub CallStaticDoubleMethodV: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *mut ()) -> jdouble,
    >,
    pub CallStaticDoubleMethodA: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methodID: jmethodID, args: *const jvalue) -> jdouble,
    >,
    pub CallStaticVoidMethod:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, cls: jclass, methodID: jmethodID, ...)>,
    pub CallStaticVoidMethodV:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, cls: jclass, methodID: jmethodID, args: *mut ())>,
    pub CallStaticVoidMethodA:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, cls: jclass, methodID: jmethodID, args: *const jvalue)>,
    pub GetStaticFieldID: ::core::option::Option<
        unsafe extern "C" fn(
            env: *mut JNIEnv,
            clazz: jclass,
            name: *const ::core::ffi::c_char,
            sig: *const ::core::ffi::c_char,
        ) -> jfieldID,
    >,
    pub GetStaticObjectField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID) -> jobject>,
    pub GetStaticBooleanField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID) -> jboolean>,
    pub GetStaticByteField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID) -> jbyte>,
    pub GetStaticCharField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID) -> jchar>,
    pub GetStaticShortField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID) -> jshort>,
    pub GetStaticIntField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID) -> jint>,
    pub GetStaticLongField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID) -> jlong>,
    pub GetStaticFloatField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID) -> jfloat>,
    pub GetStaticDoubleField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID) -> jdouble>,
    pub SetStaticObjectField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID, value: jobject)>,
    pub SetStaticBooleanField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID, value: jboolean)>,
    pub SetStaticByteField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID, value: jbyte)>,
    pub SetStaticCharField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID, value: jchar)>,
    pub SetStaticShortField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID, value: jshort)>,
    pub SetStaticIntField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID, value: jint)>,
    pub SetStaticLongField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID, value: jlong)>,
    pub SetStaticFloatField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID, value: jfloat)>,
    pub SetStaticDoubleField:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, fieldID: jfieldID, value: jdouble)>,
    pub NewString: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, unicode: *const jchar, len: jsize) -> jstring>,
    pub GetStringLength: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, str_: jstring) -> jsize>,
    pub GetStringChars:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, str_: jstring, isCopy: *mut jboolean) -> *const jchar>,
    pub ReleaseStringChars: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, str_: jstring, chars: *const jchar)>,
    pub NewStringUTF: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, utf: *const ::core::ffi::c_char) -> jstring>,
    pub GetStringUTFLength: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, str_: jstring) -> jsize>,
    pub GetStringUTFChars: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, str_: jstring, isCopy: *mut jboolean) -> *const ::core::ffi::c_char,
    >,
    pub ReleaseStringUTFChars:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, str_: jstring, chars: *const ::core::ffi::c_char)>,
    pub GetArrayLength: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, array: jarray) -> jsize>,
    pub NewObjectArray:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, len: jsize, clazz: jclass, init: jobject) -> jobjectArray>,
    pub GetObjectArrayElement:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, array: jobjectArray, index: jsize) -> jobject>,
    pub SetObjectArrayElement:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, array: jobjectArray, index: jsize, val: jobject)>,
    pub NewBooleanArray: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, len: jsize) -> jbooleanArray>,
    pub NewByteArray: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, len: jsize) -> jbyteArray>,
    pub NewCharArray: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, len: jsize) -> jcharArray>,
    pub NewShortArray: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, len: jsize) -> jshortArray>,
    pub NewIntArray: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, len: jsize) -> jintArray>,
    pub NewLongArray: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, len: jsize) -> jlongArray>,
    pub NewFloatArray: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, len: jsize) -> jfloatArray>,
    pub NewDoubleArray: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, len: jsize) -> jdoubleArray>,
    pub GetBooleanArrayElements: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, array: jbooleanArray, isCopy: *mut jboolean) -> *mut jboolean,
    >,
    pub GetByteArrayElements:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, array: jbyteArray, isCopy: *mut jboolean) -> *mut jbyte>,
    pub GetCharArrayElements:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, array: jcharArray, isCopy: *mut jboolean) -> *mut jchar>,
    pub GetShortArrayElements:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, array: jshortArray, isCopy: *mut jboolean) -> *mut jshort>,
    pub GetIntArrayElements:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, array: jintArray, isCopy: *mut jboolean) -> *mut jint>,
    pub GetLongArrayElements:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, array: jlongArray, isCopy: *mut jboolean) -> *mut jlong>,
    pub GetFloatArrayElements:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, array: jfloatArray, isCopy: *mut jboolean) -> *mut jfloat>,
    pub GetDoubleArrayElements: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, array: jdoubleArray, isCopy: *mut jboolean) -> *mut jdouble,
    >,
    pub ReleaseBooleanArrayElements:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, array: jbooleanArray, elems: *mut jboolean, mode: jint)>,
    pub ReleaseByteArrayElements:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, array: jbyteArray, elems: *mut jbyte, mode: jint)>,
    pub ReleaseCharArrayElements:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, array: jcharArray, elems: *mut jchar, mode: jint)>,
    pub ReleaseShortArrayElements:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, array: jshortArray, elems: *mut jshort, mode: jint)>,
    pub ReleaseIntArrayElements:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, array: jintArray, elems: *mut jint, mode: jint)>,
    pub ReleaseLongArrayElements:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, array: jlongArray, elems: *mut jlong, mode: jint)>,
    pub ReleaseFloatArrayElements:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, array: jfloatArray, elems: *mut jfloat, mode: jint)>,
    pub ReleaseDoubleArrayElements:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, array: jdoubleArray, elems: *mut jdouble, mode: jint)>,
    pub GetBooleanArrayRegion: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, array: jbooleanArray, start: jsize, l: jsize, buf: *mut jboolean),
    >,
    pub GetByteArrayRegion: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, array: jbyteArray, start: jsize, len: jsize, buf: *mut jbyte),
    >,
    pub GetCharArrayRegion: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, array: jcharArray, start: jsize, len: jsize, buf: *mut jchar),
    >,
    pub GetShortArrayRegion: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, array: jshortArray, start: jsize, len: jsize, buf: *mut jshort),
    >,
    pub GetIntArrayRegion: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, array: jintArray, start: jsize, len: jsize, buf: *mut jint),
    >,
    pub GetLongArrayRegion: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, array: jlongArray, start: jsize, len: jsize, buf: *mut jlong),
    >,
    pub GetFloatArrayRegion: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, array: jfloatArray, start: jsize, len: jsize, buf: *mut jfloat),
    >,
    pub GetDoubleArrayRegion: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, array: jdoubleArray, start: jsize, len: jsize, buf: *mut jdouble),
    >,
    pub SetBooleanArrayRegion: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, array: jbooleanArray, start: jsize, l: jsize, buf: *const jboolean),
    >,
    pub SetByteArrayRegion: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, array: jbyteArray, start: jsize, len: jsize, buf: *const jbyte),
    >,
    pub SetCharArrayRegion: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, array: jcharArray, start: jsize, len: jsize, buf: *const jchar),
    >,
    pub SetShortArrayRegion: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, array: jshortArray, start: jsize, len: jsize, buf: *const jshort),
    >,
    pub SetIntArrayRegion: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, array: jintArray, start: jsize, len: jsize, buf: *const jint),
    >,
    pub SetLongArrayRegion: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, array: jlongArray, start: jsize, len: jsize, buf: *const jlong),
    >,
    pub SetFloatArrayRegion: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, array: jfloatArray, start: jsize, len: jsize, buf: *const jfloat),
    >,
    pub SetDoubleArrayRegion: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, array: jdoubleArray, start: jsize, len: jsize, buf: *const jdouble),
    >,
    pub RegisterNatives: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass, methods: *const JNINativeMethod, nMethods: jint) -> jint,
    >,
    pub UnregisterNatives: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass) -> jint>,
    pub MonitorEnter: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject) -> jint>,
    pub MonitorExit: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject) -> jint>,
    pub GetJavaVM: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, vm: *mut *mut JavaVM) -> jint>,
    pub GetStringRegion:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, str_: jstring, start: jsize, len: jsize, buf: *mut jchar)>,
    pub GetStringUTFRegion: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, str_: jstring, start: jsize, len: jsize, buf: *mut ::core::ffi::c_char),
    >,
    pub GetPrimitiveArrayCritical: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, array: jarray, isCopy: *mut jboolean) -> *mut ::core::ffi::c_void,
    >,
    pub ReleasePrimitiveArrayCritical: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, array: jarray, carray: *mut ::core::ffi::c_void, mode: jint),
    >,
    pub GetStringCritical:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, string: jstring, isCopy: *mut jboolean) -> *const jchar>,
    pub ReleaseStringCritical:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, string: jstring, cstring: *const jchar)>,
    pub NewWeakGlobalRef: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject) -> jweak>,
    pub DeleteWeakGlobalRef: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, ref_: jweak)>,
    pub ExceptionCheck: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv) -> jboolean>,
    pub NewDirectByteBuffer: ::core::option::Option<
        unsafe extern "C" fn(env: *mut JNIEnv, address: *mut ::core::ffi::c_void, capacity: jlong) -> jobject,
    >,
    pub GetDirectBufferAddress:
        ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, buf: jobject) -> *mut ::core::ffi::c_void>,
    pub GetDirectBufferCapacity: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, buf: jobject) -> jlong>,
    pub GetObjectRefType: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject) -> jobjectRefType>,
    pub GetModule: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, clazz: jclass) -> jobject>,
    pub IsVirtualThread: ::core::option::Option<unsafe extern "C" fn(env: *mut JNIEnv, obj: jobject) -> jboolean>,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct JNIEnv_ {
    pub functions: *const JNINativeInterface_,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct JavaVMOption {
    pub optionString: *mut ::core::ffi::c_char,
    pub extraInfo: *mut ::core::ffi::c_void,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct JavaVMInitArgs {
    pub version: jint,
    pub nOptions: jint,
    pub options: *mut JavaVMOption,
    pub ignoreUnrecognized: jboolean,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct JavaVMAttachArgs {
    pub version: jint,
    pub name: *mut ::core::ffi::c_char,
    pub group: jobject,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct JNIInvokeInterface_ {
    pub reserved0: *mut ::core::ffi::c_void,
    pub reserved1: *mut ::core::ffi::c_void,
    pub reserved2: *mut ::core::ffi::c_void,
    pub DestroyJavaVM: ::core::option::Option<unsafe extern "C" fn(vm: *mut JavaVM) -> jint>,
    pub AttachCurrentThread: ::core::option::Option<
        unsafe extern "C" fn(vm: *mut JavaVM, penv: *mut *mut ::core::ffi::c_void, args: *mut ::core::ffi::c_void) -> jint,
    >,
    pub DetachCurrentThread: ::core::option::Option<unsafe extern "C" fn(vm: *mut JavaVM) -> jint>,
    pub GetEnv:
        ::core::option::Option<unsafe extern "C" fn(vm: *mut JavaVM, penv: *mut *mut ::core::ffi::c_void, version: jint) -> jint>,
    pub AttachCurrentThreadAsDaemon: ::core::option::Option<
        unsafe extern "C" fn(vm: *mut JavaVM, penv: *mut *mut ::core::ffi::c_void, args: *mut ::core::ffi::c_void) -> jint,
    >,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct JavaVM_ {
    pub functions: *const JNIInvokeInterface_,
}
unsafe extern "C" {
    pub fn JNI_GetDefaultJavaVMInitArgs(args: *mut ::core::ffi::c_void) -> jint;
}
unsafe extern "C" {
    pub fn JNI_CreateJavaVM(pvm: *mut *mut JavaVM, penv: *mut *mut ::core::ffi::c_void, args: *mut ::core::ffi::c_void) -> jint;
}
unsafe extern "C" {
    pub fn JNI_GetCreatedJavaVMs(arg1: *mut *mut JavaVM, arg2: jsize, arg3: *mut jsize) -> jint;
}
unsafe extern "C" {
    pub fn JNI_OnLoad(vm: *mut JavaVM, reserved: *mut ::core::ffi::c_void) -> jint;
}
unsafe extern "C" {
    pub fn JNI_OnUnload(vm: *mut JavaVM, reserved: *mut ::core::ffi::c_void);
}
