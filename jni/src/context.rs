#![allow(dead_code)]

use alloc::{string::String, vec::Vec};
use core::{ffi::CStr, marker::PhantomData, mem::MaybeUninit, ptr::null_mut};

use crate::{
    reference::{Local, Ref, StrongRef},
    sys::{jfieldID, jmethodID, jobject, jvalue, jweak, JNIEnv, JNINativeMethod, JNI_FALSE, JNI_OK, JNI_VERSION_1_4},
    throwable::JavaThrowable,
    vm, FromRaw,
};

macro_rules! call {
    ($this:ident, $func_name:ident) => {
        { $this.run(|| { ((*($this.env)).$func_name.unwrap())(&$this.env as *const _ as *mut _) }) }
    };
    ($this:ident, $func_name:ident, $($args:expr),*) => {
        { $this.run(|| { ((*($this.env)).$func_name.unwrap())(&$this.env as *const _ as *mut _, $($args),*) }) }
    };
}

macro_rules! call_nothrow {
    ($this:ident, $func_name:ident) => {
        { $this.run_no_throw(|| { ((*($this.env)).$func_name.unwrap())(&$this.env as *const _ as *mut _) }) }
    };
    ($this:ident, $func_name:ident, $($args:expr),*) => {
        { $this.run_no_throw(|| { ((*($this.env)).$func_name.unwrap())(&$this.env as *const _ as *mut _, $($args),*) }) }
    };
}

#[repr(transparent)]
pub struct Context {
    env: JNIEnv,
}

impl Context {
    pub unsafe fn throw<R: StrongRef>(&self, throwable: &R) {
        unsafe { (*self.env).Throw.unwrap()(self.as_raw(), throwable.as_raw()) };
    }

    fn run<R>(&self, f: impl FnOnce() -> R) -> Result<R, JavaThrowable<false, Local>> {
        unsafe {
            let ex = (*self.env).ExceptionOccurred.unwrap()(self.as_raw());
            if !ex.is_null() {
                (*self.env).ExceptionClear.unwrap()(self.as_raw());
            }

            let ret = f();

            let ret_ex = (*self.env).ExceptionOccurred.unwrap()(self.as_raw());
            let ret = if !ret_ex.is_null() {
                #[cfg(debug_assertions)]
                (*self.env).ExceptionDescribe.unwrap()(self.as_raw());

                (*self.env).ExceptionClear.unwrap()(self.as_raw());

                Err(JavaThrowable::from_raw(Local::from_raw(self, ret_ex)))
            } else {
                Ok(ret)
            };

            if !ex.is_null() {
                (*self.env).Throw.unwrap()(self.as_raw(), ex);

                (*self.env).DeleteLocalRef.unwrap()(self.as_raw(), ex);
            }

            ret
        }
    }

    fn run_no_throw<R>(&self, f: impl FnOnce() -> R) -> R {
        self.run(f).expect("BROKEN: jvm throw unexpected exception.")
    }

    pub fn ensure_local_capacity(&self, capacity: i32) {
        unsafe {
            call_nothrow!(self, EnsureLocalCapacity, capacity);
        }
    }
}

impl Context {
    pub unsafe fn from_raw<'a>(env: *mut JNIEnv) -> &'a Self {
        core::mem::transmute(env)
    }

    pub fn as_raw(&self) -> *mut JNIEnv {
        &self.env as *const _ as *mut _
    }
}

fn current_context<'a>() -> Option<&'a Context> {
    let vm = vm::require_vm();
    let mut env: *mut JNIEnv = null_mut();

    unsafe {
        let ret = (**vm).GetEnv?(vm, (&mut env as *mut *mut JNIEnv).cast(), JNI_VERSION_1_4 as i32);
        if ret == JNI_OK {
            Some(Context::from_raw(env))
        } else {
            None
        }
    }
}

impl Context {
    pub fn with_current<R>(f: impl FnOnce(&Self) -> R) -> Option<R> {
        let ctx = current_context()?;

        Some(f(&ctx))
    }

    pub fn with_attached<R>(f: impl FnOnce(&Self) -> R) -> R {
        match current_context() {
            None => unsafe {
                let vm = vm::require_vm();
                let mut env: *mut JNIEnv = null_mut();

                let attached = (**vm).AttachCurrentThread.unwrap()(vm, (&mut env as *mut *mut JNIEnv).cast(), null_mut());
                if attached == JNI_OK {
                    let ctx = Context::from_raw(env);

                    let ret = f(&ctx);

                    (**vm).DetachCurrentThread.unwrap()(vm);

                    ret
                } else {
                    panic!("BROKEN: unable to attach current thread.")
                }
            },
            Some(ctx) => f(&ctx),
        }
    }
}

impl Context {
    pub fn new_string(&self, s: impl AsRef<str>) -> Local {
        unsafe {
            self.ensure_local_capacity(4);

            let u16s = s.as_ref().encode_utf16().collect::<Vec<_>>();

            let obj = call_nothrow!(self, NewString, u16s.as_ptr(), u16s.len() as _);

            Local::from_raw(self, obj)
        }
    }

    pub unsafe fn get_string<R: StrongRef>(&self, s: &R) -> String {
        unsafe {
            let obj = s.as_raw();

            let length = call_nothrow!(self, GetStringLength, obj);
            let ptr = call_nothrow!(self, GetStringChars, obj, null_mut());

            let ret = String::from_utf16(core::slice::from_raw_parts(ptr, length as _))
                .expect("BROKEN: Jvm returns invalid UTF-16 string.");

            call_nothrow!(self, ReleaseStringChars, obj, ptr);

            ret
        }
    }

    pub fn get_object_class<R: StrongRef>(&self, object: &R) -> Local {
        self.ensure_local_capacity(4);

        unsafe { Local::from_raw(self, call_nothrow!(self, GetObjectClass, object.as_raw())) }
    }

    pub unsafe fn is_instance_of<R1: Ref, R2: Ref>(&self, object: &R1, class: &R2) -> bool {
        unsafe { call_nothrow!(self, IsInstanceOf, object.as_raw(), class.as_raw()) != JNI_FALSE }
    }

    pub unsafe fn is_assignable_from<R1: Ref, R2: Ref>(&self, class: &R1, superclass: &R2) -> bool {
        unsafe { call_nothrow!(self, IsAssignableFrom, class.as_raw(), superclass.as_raw()) != JNI_FALSE }
    }

    pub fn is_same_object<R1: Ref, R2: Ref>(&self, a: Option<&R1>, b: Option<&R2>) -> bool {
        unsafe {
            call_nothrow!(
                self,
                IsSameObject,
                a.map(|r| r.as_raw()).unwrap_or(null_mut()),
                b.map(|r| r.as_raw()).unwrap_or(null_mut())
            ) != JNI_FALSE
        }
    }
}

impl Context {
    pub unsafe fn new_global_ref(&self, object: jobject) -> jobject {
        call_nothrow!(self, NewGlobalRef, object)
    }

    pub unsafe fn new_local_ref(&self, object: jobject) -> jobject {
        self.ensure_local_capacity(4);

        call_nothrow!(self, NewLocalRef, object)
    }

    pub unsafe fn new_weak_global_ref(&self, object: jobject) -> jweak {
        call_nothrow!(self, NewWeakGlobalRef, object)
    }

    pub unsafe fn delete_global_ref(&self, object: jobject) {
        call_nothrow!(self, DeleteGlobalRef, object)
    }

    pub unsafe fn delete_local_ref(&self, object: jobject) {
        call_nothrow!(self, DeleteLocalRef, object)
    }

    pub unsafe fn delete_weak_global_ref(&self, object: jweak) {
        call_nothrow!(self, DeleteWeakGlobalRef, object)
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Method<const STATIC: bool>(jmethodID);

unsafe impl<const STATIC: bool> Send for Method<STATIC> {}
unsafe impl<const STATIC: bool> Sync for Method<STATIC> {}

impl<const STATIC: bool> Method<STATIC> {
    pub unsafe fn from_raw(raw: jmethodID) -> Self {
        Self(raw)
    }

    pub fn as_raw(&self) -> jmethodID {
        self.0
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Field<const STATIC: bool>(jfieldID);

unsafe impl<const STATIC: bool> Send for Field<STATIC> {}
unsafe impl<const STATIC: bool> Sync for Field<STATIC> {}

impl<const STATIC: bool> Field<STATIC> {
    pub unsafe fn from_raw(raw: jfieldID) -> Self {
        Self(raw)
    }

    pub fn as_raw(&self) -> jfieldID {
        self.0
    }
}

impl Context {
    pub fn find_class(&self, name: impl AsRef<CStr>) -> Result<Local, JavaThrowable<false, Local>> {
        unsafe { call!(self, FindClass, name.as_ref().as_ptr()).map(|r| Local::from_raw(self, r)) }
    }

    pub fn find_method<const STATIC: bool, C: StrongRef>(
        &self,
        class: &C,
        name: impl AsRef<CStr>,
        signature: impl AsRef<CStr>,
    ) -> Result<Method<STATIC>, JavaThrowable<false, Local>> {
        unsafe {
            let raw = if STATIC {
                call!(
                    self,
                    GetStaticMethodID,
                    class.as_raw(),
                    name.as_ref().as_ptr(),
                    signature.as_ref().as_ptr()
                )
            } else {
                call!(
                    self,
                    GetMethodID,
                    class.as_raw(),
                    name.as_ref().as_ptr(),
                    signature.as_ref().as_ptr()
                )
            };

            raw.map(|id| Method::from_raw(id))
        }
    }

    pub fn find_field<const STATIC: bool, C: StrongRef>(
        &self,
        class: &C,
        name: impl AsRef<CStr>,
        signature: impl AsRef<CStr>,
    ) -> Result<Field<STATIC>, JavaThrowable<false, Local>> {
        unsafe {
            let raw = if STATIC {
                call!(
                    self,
                    GetStaticFieldID,
                    class.as_raw(),
                    name.as_ref().as_ptr(),
                    signature.as_ref().as_ptr()
                )
            } else {
                call!(
                    self,
                    GetFieldID,
                    class.as_raw(),
                    name.as_ref().as_ptr(),
                    signature.as_ref().as_ptr()
                )
            };

            raw.map(|id| Field::from_raw(id))
        }
    }
}

#[derive(Clone)]
pub struct AnyObject<'a> {
    raw: jobject,
    _ref: PhantomData<&'a jobject>,
}

#[derive(Clone)]
pub enum CallArg<'a> {
    Boolean(bool),
    Byte(i8),
    Char(u16),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Object(Option<AnyObject<'a>>),
}

impl<'a> CallArg<'a> {
    fn as_raw(&self) -> jvalue {
        match self {
            CallArg::Boolean(z) => jvalue { z: *z },
            CallArg::Byte(b) => jvalue { b: *b },
            CallArg::Char(c) => jvalue { c: *c },
            CallArg::Short(s) => jvalue { s: *s },
            CallArg::Int(i) => jvalue { i: *i },
            CallArg::Long(j) => jvalue { j: *j },
            CallArg::Float(f) => jvalue { f: *f },
            CallArg::Double(d) => jvalue { d: *d },
            CallArg::Object(Some(obj)) => jvalue { l: obj.raw },
            CallArg::Object(None) => jvalue { l: null_mut() },
        }
    }
}

macro_rules! impl_value_from {
    ($typ:ty, $variant:ident) => {
        impl<'a> From<$typ> for CallArg<'a> {
            fn from(value: $typ) -> Self {
                Self::$variant(value)
            }
        }
    };
}

impl_value_from!(bool, Boolean);
impl_value_from!(i8, Byte);
impl_value_from!(u16, Char);
impl_value_from!(i16, Short);
impl_value_from!(i32, Int);
impl_value_from!(i64, Long);
impl_value_from!(f32, Float);
impl_value_from!(f64, Double);

impl<'a, R: Ref> From<&'a R> for CallArg<'a> {
    fn from(value: &'a R) -> Self {
        Self::Object(Some(AnyObject {
            raw: value.as_raw(),
            _ref: PhantomData,
        }))
    }
}

impl<'a, R: Ref> From<Option<&'a R>> for CallArg<'a> {
    fn from(value: Option<&'a R>) -> Self {
        Self::Object(value.map(|r| AnyObject {
            raw: r.as_raw(),
            _ref: PhantomData,
        }))
    }
}

#[doc(hidden)]
pub trait CallResult<'ctx>: Sized + 'ctx {
    unsafe fn call<const STATIC: bool, T: StrongRef>(
        ctx: &'ctx Context,
        this: &T,
        method: Method<STATIC>,
        args: &[jvalue],
    ) -> Result<Self, JavaThrowable<false, Local<'ctx>>>;
}

macro_rules! impl_call_result {
    ($typ:ty, $call:ident, $call_static:ident) => {
        impl<'ctx> CallResult<'ctx> for $typ {
            unsafe fn call<const STATIC: bool, T: StrongRef>(
                ctx: &'ctx Context,
                this: &T,
                method: Method<STATIC>,
                args: &[jvalue],
            ) -> Result<Self, JavaThrowable<false, Local<'ctx>>> {
                if STATIC {
                    call!(ctx, $call_static, this.as_raw(), method.as_raw(), args.as_ptr())
                } else {
                    call!(ctx, $call, this.as_raw(), method.as_raw(), args.as_ptr())
                }
            }
        }
    };
}

impl_call_result!((), CallVoidMethodA, CallStaticVoidMethodA);
impl_call_result!(bool, CallBooleanMethodA, CallStaticBooleanMethodA);
impl_call_result!(i8, CallByteMethodA, CallStaticByteMethodA);
impl_call_result!(u16, CallCharMethodA, CallStaticCharMethodA);
impl_call_result!(i16, CallShortMethodA, CallStaticShortMethodA);
impl_call_result!(i32, CallIntMethodA, CallStaticIntMethodA);
impl_call_result!(i64, CallLongMethodA, CallStaticLongMethodA);
impl_call_result!(f32, CallFloatMethodA, CallStaticFloatMethodA);
impl_call_result!(f64, CallDoubleMethodA, CallStaticDoubleMethodA);

impl<'ctx> CallResult<'ctx> for Option<Local<'ctx>> {
    unsafe fn call<const STATIC: bool, T: StrongRef>(
        ctx: &'ctx Context,
        this: &T,
        method: Method<STATIC>,
        args: &[jvalue],
    ) -> Result<Self, JavaThrowable<false, Local<'ctx>>> {
        let ret = if STATIC {
            call!(ctx, CallStaticObjectMethodA, this.as_raw(), method.as_raw(), args.as_ptr())
        } else {
            call!(ctx, CallObjectMethodA, this.as_raw(), method.as_raw(), args.as_ptr())
        };

        ret.map(|o| if o.is_null() { None } else { Some(Local::from_raw(ctx, o)) })
    }
}

#[doc(hidden)]
pub trait CallArgs {
    type RawArgs: AsRef<[jvalue]>;

    fn as_raw(&self) -> Self::RawArgs;
}

impl<'a, const N: usize> CallArgs for [CallArg<'a>; N] {
    type RawArgs = [jvalue; N];

    fn as_raw(&self) -> Self::RawArgs {
        unsafe {
            let mut ret = [MaybeUninit::<jvalue>::uninit(); N];

            for (index, value) in self.iter().enumerate() {
                ret[index] = MaybeUninit::new(value.as_raw());
            }

            ret.map(|m| m.assume_init())
        }
    }
}

impl<'a> CallArgs for &[CallArg<'a>] {
    type RawArgs = Vec<jvalue>;

    fn as_raw(&self) -> Self::RawArgs {
        self.iter().map(|v| v.as_raw()).collect()
    }
}

impl Context {
    pub unsafe fn new_object<'ctx, R: StrongRef, A: CallArgs>(
        &self,
        class: &R,
        method: Method<false>,
        args: A,
    ) -> Result<Local, JavaThrowable<false, Local>> {
        self.ensure_local_capacity(4);

        let args = args.as_raw();
        let args = args.as_ref();

        call!(self, NewObjectA, class.as_raw(), method.as_raw(), args.as_ptr()).map(|r| Local::from_raw(self, r))
    }

    pub unsafe fn call_method<'ctx, const STATIC: bool, T: StrongRef, A: CallArgs, R: CallResult<'ctx>>(
        &'ctx self,
        this: &T,
        method: Method<STATIC>,
        args: A,
    ) -> Result<R, JavaThrowable<false, Local<'ctx>>> {
        let args = args.as_raw();
        let args = args.as_ref();

        R::call(self, this, method, args)
    }
}

#[doc(hidden)]
pub trait GetReturn<'ctx>: Sized + 'ctx {
    unsafe fn get<const STATIC: bool, T: StrongRef>(ctx: &'ctx Context, this: &T, field: Field<STATIC>) -> Self;
}

#[doc(hidden)]
pub trait SetArg: Sized {
    unsafe fn set<const STATIC: bool, T: StrongRef>(self, ctx: &Context, this: &T, field: Field<STATIC>);
}

macro_rules! impl_get_return {
    ($typ:ty, $get:ident, $get_static:ident) => {
        impl<'ctx> GetReturn<'ctx> for $typ {
            unsafe fn get<const STATIC: bool, T: StrongRef>(ctx: &'ctx Context, this: &T, field: Field<STATIC>) -> Self {
                if STATIC {
                    call_nothrow!(ctx, $get_static, this.as_raw(), field.as_raw())
                } else {
                    call_nothrow!(ctx, $get, this.as_raw(), field.as_raw())
                }
            }
        }
    };
}

macro_rules! impl_set_arg {
    ($typ:ty, $set:ident, $set_static:ident) => {
        impl SetArg for $typ {
            unsafe fn set<const STATIC: bool, T: StrongRef>(self, ctx: &Context, this: &T, field: Field<STATIC>) {
                if STATIC {
                    call_nothrow!(ctx, $set_static, this.as_raw(), field.as_raw(), self)
                } else {
                    call_nothrow!(ctx, $set, this.as_raw(), field.as_raw(), self)
                }
            }
        }
    };
}

impl_get_return!(bool, GetBooleanField, GetStaticBooleanField);
impl_get_return!(i8, GetByteField, GetStaticByteField);
impl_get_return!(u16, GetCharField, GetStaticCharField);
impl_get_return!(i16, GetShortField, GetStaticShortField);
impl_get_return!(i32, GetIntField, GetStaticIntField);
impl_get_return!(i64, GetLongField, GetStaticLongField);
impl_get_return!(f32, GetFloatField, GetStaticFloatField);
impl_get_return!(f64, GetDoubleField, GetStaticDoubleField);

impl_set_arg!(bool, SetBooleanField, SetStaticBooleanField);
impl_set_arg!(i8, SetByteField, SetStaticByteField);
impl_set_arg!(u16, SetCharField, SetStaticCharField);
impl_set_arg!(i16, SetShortField, SetStaticShortField);
impl_set_arg!(i32, SetIntField, SetStaticIntField);
impl_set_arg!(i64, SetLongField, SetStaticLongField);
impl_set_arg!(f32, SetFloatField, SetStaticFloatField);
impl_set_arg!(f64, SetDoubleField, SetStaticDoubleField);

impl<'ctx> GetReturn<'ctx> for Option<Local<'ctx>> {
    unsafe fn get<const STATIC: bool, T: StrongRef>(ctx: &'ctx Context, this: &T, field: Field<STATIC>) -> Self {
        let ret = if STATIC {
            call_nothrow!(ctx, GetStaticObjectField, this.as_raw(), field.as_raw())
        } else {
            call_nothrow!(ctx, GetObjectField, this.as_raw(), field.as_raw())
        };

        if ret.is_null() {
            None
        } else {
            Some(Local::from_raw(ctx, ret))
        }
    }
}

impl<'a> SetArg for &'a Local<'a> {
    unsafe fn set<const STATIC: bool, T: StrongRef>(self, ctx: &Context, this: &T, field: Field<STATIC>) {
        if STATIC {
            call_nothrow!(ctx, SetStaticObjectField, this.as_raw(), field.as_raw(), self.as_raw())
        } else {
            call_nothrow!(ctx, SetObjectField, this.as_raw(), field.as_raw(), self.as_raw())
        }
    }
}

impl<'a> SetArg for Local<'a> {
    unsafe fn set<const STATIC: bool, T: StrongRef>(self, ctx: &Context, this: &T, field: Field<STATIC>) {
        (&self).set(ctx, this, field)
    }
}

impl<'a> SetArg for Option<&'a Local<'a>> {
    unsafe fn set<const STATIC: bool, T: StrongRef>(self, ctx: &Context, this: &T, field: Field<STATIC>) {
        match self {
            None => {
                if STATIC {
                    call_nothrow!(ctx, SetStaticObjectField, this.as_raw(), field.as_raw(), null_mut())
                } else {
                    call_nothrow!(ctx, SetObjectField, this.as_raw(), field.as_raw(), null_mut())
                }
            }
            Some(l) => l.set(ctx, this, field),
        }
    }
}

impl Context {
    pub unsafe fn get_field<'ctx, const STATIC: bool, T: StrongRef, V: GetReturn<'ctx>>(
        &'ctx self,
        this: &T,
        field: Field<STATIC>,
    ) -> V {
        V::get(self, this, field)
    }

    pub unsafe fn set_field<const STATIC: bool, T: StrongRef, V: SetArg>(&self, this: &T, field: Field<STATIC>, value: V) {
        value.set(self, this, field)
    }
}

#[doc(hidden)]
pub trait PrimitiveArrayElement: Sized {
    unsafe fn new_array(ctx: &Context, length: i32) -> Result<Local, JavaThrowable<false, Local>>;

    unsafe fn get_region<'ctx, T: StrongRef>(
        ctx: &'ctx Context,
        this: &T,
        offset: i32,
        buf: &mut [Self],
    ) -> Result<(), JavaThrowable<false, Local<'ctx>>>;

    unsafe fn set_region<'ctx, T: StrongRef>(
        ctx: &'ctx Context,
        this: &T,
        offset: i32,
        buf: &[Self],
    ) -> Result<(), JavaThrowable<false, Local<'ctx>>>;
}

macro_rules! impl_primitive_array_element {
    ($typ:ty, $new:ident, $get_region:ident, $set_region:ident) => {
        impl PrimitiveArrayElement for $typ {
            unsafe fn new_array(ctx: &Context, length: i32) -> Result<Local, JavaThrowable<false, Local>> {
                call!(ctx, $new, length).map(|r| Local::from_raw(ctx, r))
            }

            unsafe fn get_region<'ctx, T: StrongRef>(
                ctx: &'ctx Context,
                this: &T,
                offset: i32,
                buf: &mut [Self],
            ) -> Result<(), JavaThrowable<false, Local<'ctx>>> {
                // TODO: throw out of bound exception instead of unwrap

                call!(
                    ctx,
                    $get_region,
                    this.as_raw(),
                    offset,
                    buf.len().try_into().unwrap(),
                    buf.as_mut_ptr()
                )
            }

            unsafe fn set_region<'ctx, T: StrongRef>(
                ctx: &'ctx Context,
                this: &T,
                offset: i32,
                buf: &[Self],
            ) -> Result<(), JavaThrowable<false, Local<'ctx>>> {
                // TODO: throw out of bound exception instead of unwrap

                call!(
                    ctx,
                    $set_region,
                    this.as_raw(),
                    offset,
                    buf.len().try_into().unwrap(),
                    buf.as_ptr()
                )
            }
        }
    };
}

impl_primitive_array_element!(bool, NewBooleanArray, GetBooleanArrayRegion, SetBooleanArrayRegion);
impl_primitive_array_element!(i8, NewByteArray, GetByteArrayRegion, SetByteArrayRegion);
impl_primitive_array_element!(u16, NewCharArray, GetCharArrayRegion, SetCharArrayRegion);
impl_primitive_array_element!(i16, NewShortArray, GetShortArrayRegion, SetShortArrayRegion);
impl_primitive_array_element!(i32, NewIntArray, GetIntArrayRegion, SetIntArrayRegion);
impl_primitive_array_element!(i64, NewLongArray, GetLongArrayRegion, SetLongArrayRegion);
impl_primitive_array_element!(f32, NewFloatArray, GetFloatArrayRegion, SetFloatArrayRegion);
impl_primitive_array_element!(f64, NewDoubleArray, GetDoubleArrayRegion, SetDoubleArrayRegion);

impl Context {
    pub unsafe fn get_array_length<R: StrongRef>(&self, object: &R) -> i32 {
        call_nothrow!(self, GetArrayLength, object.as_raw())
    }

    pub unsafe fn new_primitive_array<E: PrimitiveArrayElement>(&self, size: i32) -> Result<Local, JavaThrowable<false, Local>> {
        E::new_array(self, size)
    }

    pub unsafe fn get_primitive_array_region<'ctx, E: PrimitiveArrayElement, T: StrongRef>(
        &'ctx self,
        this: &T,
        offset: i32,
        buf: &mut [E],
    ) -> Result<(), JavaThrowable<false, Local<'ctx>>> {
        E::get_region(self, this, offset, buf)
    }

    pub unsafe fn set_primitive_array_region<'ctx, E: PrimitiveArrayElement, T: StrongRef>(
        &'ctx self,
        this: &T,
        offset: i32,
        buf: &[E],
    ) -> Result<(), JavaThrowable<false, Local<'ctx>>> {
        E::set_region(self, this, offset, buf)
    }

    pub unsafe fn new_object_array<R1: StrongRef, R2: Ref>(
        &self,
        length: i32,
        class: &R1,
        initial: Option<&R2>,
    ) -> Result<Local, JavaThrowable<false, Local>> {
        self.ensure_local_capacity(4);

        call!(
            self,
            NewObjectArray,
            length,
            class.as_raw(),
            initial.map(|r| r.as_raw()).unwrap_or(null_mut())
        )
        .map(|r| Local::from_raw(self, r))
    }

    pub unsafe fn get_object_array_element<R: StrongRef>(
        &self,
        object: &R,
        index: i32,
    ) -> Result<Option<Local>, JavaThrowable<false, Local>> {
        self.ensure_local_capacity(4);

        call!(self, GetObjectArrayElement, object.as_raw(), index).map(|raw| {
            if raw.is_null() {
                None
            } else {
                Some(Local::from_raw(self, raw))
            }
        })
    }

    pub unsafe fn set_object_array_element<R1: StrongRef, R2: Ref>(
        &self,
        object: &R1,
        index: i32,
        value: Option<&R2>,
    ) -> Result<(), JavaThrowable<false, Local>> {
        call!(
            self,
            SetObjectArrayElement,
            object.as_raw(),
            index,
            value.map(|r| r.as_raw()).unwrap_or(null_mut())
        )
    }
}

impl Context {
    pub unsafe fn register_natives<const COUNT: usize, R: StrongRef, N: AsRef<CStr>, S: AsRef<CStr>>(
        &self,
        class: &R,
        natives: [(N, S, *const ()); COUNT],
    ) -> Result<(), JavaThrowable<false, Local>> {
        unsafe {
            let mut funcs = [MaybeUninit::<JNINativeMethod>::uninit(); COUNT];
            for (index, (name, signature, ptr)) in natives.iter().enumerate() {
                funcs[index] = MaybeUninit::new(JNINativeMethod {
                    name: name.as_ref().as_ptr() as _,
                    signature: signature.as_ref().as_ptr() as _,
                    fnPtr: *ptr as _,
                });
            }

            let n = call!(self, RegisterNatives, class.as_raw(), funcs.as_ptr() as _, funcs.len() as _)?;
            if n != JNI_OK {
                panic!("BROKEN: register native to jvm failed")
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_reference_equal_ptr() {
    let raw = 0x123456789usize as *mut JNIEnv;
    let ctx = unsafe { Context::from_raw(raw) };
    let r_raw = ctx.as_raw();

    assert_eq!(raw, r_raw);
}