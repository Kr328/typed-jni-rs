#![allow(dead_code)]

use alloc::{string::String, vec::Vec};
use core::{
    ffi::CStr,
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::{NonNull, null_mut},
};

use crate::{
    raw::{AsRaw, FromRaw, IntoRaw, Raw},
    reference::{Global, Local, Ref, StrongRef, Weak},
    sys::{
        JNI_ABORT, JNI_COMMIT, JNI_FALSE, JNI_OK, JNIEnv, JNINativeInterface_, JNINativeMethod, jfieldID, jmethodID, jobject,
        jvalue,
    },
    vm,
};

mod __sealed {
    pub trait Sealed {}
}

macro_rules! call {
    ($this:ident, $func_name:ident) => {
        { $this.run(|| { $this.env.as_ref().$func_name.unwrap()(&$this.env as *const _ as *mut _) }) }
    };
    ($this:ident, $func_name:ident, $($args:expr),*) => {
        { $this.run(|| { $this.env.as_ref().$func_name.unwrap()(&$this.env as *const _ as *mut _, $($args),*) }) }
    };
}

macro_rules! call_nothrow {
    ($this:ident, $func_name:ident) => {
        { $this.run_no_throw(|| { $this.env.as_ref().$func_name.unwrap()(&$this.env as *const _ as *mut _) }) }
    };
    ($this:ident, $func_name:ident, $($args:expr),*) => {
        { $this.run_no_throw(|| { $this.env.as_ref().$func_name.unwrap()(&$this.env as *const _ as *mut _, $($args),*) }) }
    };
}

#[repr(transparent)]
pub struct Context {
    env: NonNull<JNINativeInterface_>,
}

impl Context {
    pub unsafe fn throw<R: StrongRef>(&self, throwable: &R) {
        unsafe { self.env.as_ref().Throw.unwrap()(self.as_raw(), *throwable.as_raw()) };
    }

    fn run<'ctx, R>(&'ctx self, f: impl FnOnce() -> R) -> Result<R, Local<'ctx>>
    where
        R: 'ctx,
    {
        unsafe {
            let ex = self.env.as_ref().ExceptionOccurred.unwrap()(self.as_raw());
            if !ex.is_null() {
                self.env.as_ref().ExceptionClear.unwrap()(self.as_raw());
            }

            let ret = f();

            let ret_ex = self.env.as_ref().ExceptionOccurred.unwrap()(self.as_raw());
            let ret = if !ret_ex.is_null() {
                #[cfg(debug_assertions)]
                self.env.as_ref().ExceptionDescribe.unwrap()(self.as_raw());

                self.env.as_ref().ExceptionClear.unwrap()(self.as_raw());

                Err(Local::from_raw(ret_ex))
            } else {
                Ok(ret)
            };

            if !ex.is_null() {
                self.env.as_ref().Throw.unwrap()(self.as_raw(), ex);

                self.env.as_ref().DeleteLocalRef.unwrap()(self.as_raw(), ex);
            }

            ret
        }
    }

    fn run_no_throw<R>(&self, f: impl FnOnce() -> R) -> R {
        self.run(f)
            .unwrap_or_else(|_| panic!("BROKEN: jvm throw unexpected exception."))
    }

    pub fn ensure_local_capacity(&self, capacity: i32) {
        unsafe {
            call_nothrow!(self, EnsureLocalCapacity, capacity);
        }
    }
}

impl Context {
    pub unsafe fn from_raw<'a>(env: *mut JNIEnv) -> &'a Self {
        unsafe { core::mem::transmute(env) }
    }

    pub fn as_raw(&self) -> *mut JNIEnv {
        &self.env as *const _ as *mut _
    }
}

impl Context {
    pub fn with_current<R>(f: impl for<'a> FnOnce(&'a Self) -> R) -> Option<R> {
        unsafe {
            let ctx = vm::current()?;

            Some(f(&Context::from_raw(ctx)))
        }
    }

    pub fn with_attached<R>(f: impl for<'a> FnOnce(&'a Self) -> R) -> R {
        unsafe {
            let attached = vm::attach();

            f(&Context::from_raw(attached.env()))
        }
    }
}

impl Context {
    pub fn new_string(&self, s: impl AsRef<str>) -> Local<'_> {
        unsafe {
            self.ensure_local_capacity(4);

            let u16s = s.as_ref().encode_utf16().collect::<Vec<_>>();

            let obj = call_nothrow!(self, NewString, u16s.as_ptr(), u16s.len() as _);

            Local::from_raw(obj)
        }
    }

    pub unsafe fn get_string<R: StrongRef>(&self, s: &R) -> String {
        unsafe {
            let obj = s.as_raw();

            let length = call_nothrow!(self, GetStringLength, *obj);
            let ptr = call_nothrow!(self, GetStringChars, *obj, null_mut());

            let ret = String::from_utf16(core::slice::from_raw_parts(ptr, length as _))
                .expect("BROKEN: Jvm returns invalid UTF-16 string.");

            call_nothrow!(self, ReleaseStringChars, *obj, ptr);

            ret
        }
    }

    pub fn get_object_class<R: StrongRef>(&self, object: &R) -> Local<'_> {
        self.ensure_local_capacity(4);

        unsafe { Local::from_raw(call_nothrow!(self, GetObjectClass, *object.as_raw())) }
    }

    pub unsafe fn is_instance_of<R1: StrongRef, R2: StrongRef>(&self, object: &R1, class: &R2) -> bool {
        unsafe { call_nothrow!(self, IsInstanceOf, *object.as_raw(), *class.as_raw()) != JNI_FALSE }
    }

    pub unsafe fn is_assignable_from<R1: StrongRef, R2: StrongRef>(&self, class: &R1, superclass: &R2) -> bool {
        unsafe { call_nothrow!(self, IsAssignableFrom, *class.as_raw(), *superclass.as_raw()) != JNI_FALSE }
    }

    pub fn is_same_object<R1: Ref, R2: Ref>(&self, a: Option<&R1>, b: Option<&R2>) -> bool {
        unsafe {
            call_nothrow!(
                self,
                IsSameObject,
                a.map(|r| *r.as_raw()).unwrap_or(null_mut()),
                b.map(|r| *r.as_raw()).unwrap_or(null_mut())
            ) != JNI_FALSE
        }
    }
}

impl Context {
    pub fn new_global_ref<R: Ref>(&self, r: &R) -> Option<Global> {
        unsafe {
            let raw = call_nothrow!(self, NewGlobalRef, *r.as_raw());

            if raw.is_null() { None } else { Some(Global::from_raw(raw)) }
        }
    }

    pub fn new_local_ref<R: Ref>(&self, r: &R) -> Option<Local<'_>> {
        unsafe {
            self.ensure_local_capacity(1);

            let raw = call_nothrow!(self, NewLocalRef, *r.as_raw());

            if raw.is_null() { None } else { Some(Local::from_raw(raw)) }
        }
    }

    pub fn new_weak_global_ref<R: Ref>(&self, r: &R) -> Option<Weak> {
        unsafe {
            let raw = call_nothrow!(self, NewWeakGlobalRef, *r.as_raw());

            if raw.is_null() { None } else { Some(Weak::from_raw(raw)) }
        }
    }
}

macro_rules! define_member {
    ($name:ident, $raw:ty) => {
        #[repr(transparent)]
        #[derive(Copy, Clone)]
        pub struct $name<const STATIC: bool>($raw);

        unsafe impl<const STATIC: bool> Send for $name<STATIC> {}
        unsafe impl<const STATIC: bool> Sync for $name<STATIC> {}

        impl<const STATIC: bool> Raw for $name<STATIC> {
            type Raw = $raw;
        }

        impl<const STATIC: bool> IntoRaw for $name<STATIC> {
            fn into_raw(self) -> Self::Raw {
                self.0
            }
        }

        impl<const STATIC: bool> AsRaw for $name<STATIC> {
            fn as_raw(&self) -> &Self::Raw {
                &self.0
            }
        }

        impl<const STATIC: bool> FromRaw for $name<STATIC> {
            unsafe fn from_raw(raw: Self::Raw) -> Self {
                Self(raw)
            }
        }
    };
}

define_member!(Method, jmethodID);
define_member!(Field, jfieldID);

impl Context {
    pub fn find_class(&self, name: impl AsRef<CStr>) -> Result<Local<'_>, Local<'_>> {
        unsafe { call!(self, FindClass, name.as_ref().as_ptr()).map(|r| Local::from_raw(r)) }
    }

    pub fn find_method<const STATIC: bool, C: StrongRef>(
        &self,
        class: &C,
        name: impl AsRef<CStr>,
        signature: impl AsRef<CStr>,
    ) -> Result<Method<STATIC>, Local<'_>> {
        unsafe {
            let raw = if STATIC {
                call!(
                    self,
                    GetStaticMethodID,
                    *class.as_raw(),
                    name.as_ref().as_ptr(),
                    signature.as_ref().as_ptr()
                )
            } else {
                call!(
                    self,
                    GetMethodID,
                    *class.as_raw(),
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
    ) -> Result<Field<STATIC>, Local<'_>> {
        unsafe {
            let raw = if STATIC {
                call!(
                    self,
                    GetStaticFieldID,
                    *class.as_raw(),
                    name.as_ref().as_ptr(),
                    signature.as_ref().as_ptr()
                )
            } else {
                call!(
                    self,
                    GetFieldID,
                    *class.as_raw(),
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
            raw: *value.as_raw(),
            _ref: PhantomData,
        }))
    }
}

impl<'a, R: Ref> From<Option<&'a R>> for CallArg<'a> {
    fn from(value: Option<&'a R>) -> Self {
        Self::Object(value.map(|r| AnyObject {
            raw: *r.as_raw(),
            _ref: PhantomData,
        }))
    }
}

#[doc(hidden)]
pub trait CallResult<'ctx>: Sized + __sealed::Sealed + 'ctx {
    unsafe fn call<const STATIC: bool, T: StrongRef>(
        ctx: &'ctx Context,
        this: &T,
        method: Method<STATIC>,
        args: &[jvalue],
    ) -> Result<Self, Local<'ctx>>;
}

macro_rules! impl_call_result {
    ($typ:ty, $call:ident, $call_static:ident) => {
        impl __sealed::Sealed for $typ {}

        impl<'ctx> CallResult<'ctx> for $typ {
            unsafe fn call<const STATIC: bool, T: StrongRef>(
                ctx: &'ctx Context,
                this: &T,
                method: Method<STATIC>,
                args: &[jvalue],
            ) -> Result<Self, Local<'ctx>> {
                unsafe {
                    if STATIC {
                        call!(ctx, $call_static, *this.as_raw(), method.into_raw(), args.as_ptr())
                    } else {
                        call!(ctx, $call, *this.as_raw(), method.into_raw(), args.as_ptr())
                    }
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

impl<'ctx> __sealed::Sealed for Option<Local<'ctx>> {}

impl<'ctx> CallResult<'ctx> for Option<Local<'ctx>> {
    unsafe fn call<const STATIC: bool, T: StrongRef>(
        ctx: &'ctx Context,
        this: &T,
        method: Method<STATIC>,
        args: &[jvalue],
    ) -> Result<Self, Local<'ctx>> {
        unsafe {
            let ret = if STATIC {
                call!(ctx, CallStaticObjectMethodA, *this.as_raw(), method.into_raw(), args.as_ptr())
            } else {
                call!(ctx, CallObjectMethodA, *this.as_raw(), method.into_raw(), args.as_ptr())
            };

            ret.map(|o| if o.is_null() { None } else { Some(Local::from_raw(o)) })
        }
    }
}

impl<'ctx> __sealed::Sealed for Local<'ctx> {}

impl<'ctx> CallResult<'ctx> for Local<'ctx> {
    unsafe fn call<const STATIC: bool, T: StrongRef>(
        ctx: &'ctx Context,
        this: &T,
        method: Method<STATIC>,
        args: &[jvalue],
    ) -> Result<Self, Local<'ctx>> {
        unsafe { Ok(Option::<Local>::call(ctx, this, method, args)?.expect("unexpected null value returns from java")) }
    }
}

#[doc(hidden)]
pub trait CallArgs: __sealed::Sealed {
    type RawArgs: AsRef<[jvalue]>;

    fn as_raw(&self) -> Self::RawArgs;
}

impl<'a, const N: usize> __sealed::Sealed for [CallArg<'a>; N] {}

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

impl<'a> __sealed::Sealed for &[CallArg<'a>] {}

impl<'a> CallArgs for &[CallArg<'a>] {
    type RawArgs = Vec<jvalue>;

    fn as_raw(&self) -> Self::RawArgs {
        self.iter().map(|v| v.as_raw()).collect()
    }
}

impl Context {
    pub unsafe fn new_object<R: StrongRef, A: CallArgs>(
        &self,
        class: &R,
        method: Method<false>,
        args: A,
    ) -> Result<Local<'_>, Local<'_>> {
        unsafe {
            self.ensure_local_capacity(4);

            let args = args.as_raw();
            let args = args.as_ref();

            call!(self, NewObjectA, *class.as_raw(), method.into_raw(), args.as_ptr()).map(|r| Local::from_raw(r))
        }
    }

    pub unsafe fn call_method<'ctx, const STATIC: bool, T: StrongRef, A: CallArgs, R: CallResult<'ctx>>(
        &'ctx self,
        this: &T,
        method: Method<STATIC>,
        args: A,
    ) -> Result<R, Local<'ctx>> {
        unsafe {
            let args = args.as_raw();
            let args = args.as_ref();

            R::call(self, this, method, args)
        }
    }
}

#[doc(hidden)]
pub trait GetReturn<'ctx>: Sized + __sealed::Sealed + 'ctx {
    unsafe fn get<const STATIC: bool, T: StrongRef>(
        ctx: &'ctx Context,
        this: &T,
        field: Field<STATIC>,
    ) -> Result<Self, Local<'ctx>>;
}

#[doc(hidden)]
pub trait SetArg: Sized + __sealed::Sealed {
    unsafe fn set<'ctx, const STATIC: bool, T: StrongRef>(
        self,
        ctx: &'ctx Context,
        this: &T,
        field: Field<STATIC>,
    ) -> Result<(), Local<'ctx>>;
}

macro_rules! impl_get_return {
    ($typ:ty, $get:ident, $get_static:ident) => {
        impl<'ctx> GetReturn<'ctx> for $typ {
            unsafe fn get<const STATIC: bool, T: StrongRef>(
                ctx: &'ctx Context,
                this: &T,
                field: Field<STATIC>,
            ) -> Result<Self, Local<'ctx>> {
                unsafe {
                    if STATIC {
                        call!(ctx, $get_static, *this.as_raw(), field.into_raw())
                    } else {
                        call!(ctx, $get, *this.as_raw(), field.into_raw())
                    }
                }
            }
        }
    };
}

macro_rules! impl_set_arg {
    ($typ:ty, $set:ident, $set_static:ident) => {
        impl SetArg for $typ {
            unsafe fn set<'ctx, const STATIC: bool, T: StrongRef>(
                self,
                ctx: &'ctx Context,
                this: &T,
                field: Field<STATIC>,
            ) -> Result<(), Local<'ctx>> {
                unsafe {
                    if STATIC {
                        call!(ctx, $set_static, *this.as_raw(), field.into_raw(), self)
                    } else {
                        call!(ctx, $set, *this.as_raw(), field.into_raw(), self)
                    }
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
    unsafe fn get<const STATIC: bool, T: StrongRef>(
        ctx: &'ctx Context,
        this: &T,
        field: Field<STATIC>,
    ) -> Result<Self, Local<'ctx>> {
        unsafe {
            let ret = if STATIC {
                call!(ctx, GetStaticObjectField, *this.as_raw(), field.into_raw())
            } else {
                call!(ctx, GetObjectField, *this.as_raw(), field.into_raw())
            };

            match ret {
                Ok(ret) => {
                    if ret.is_null() {
                        Ok(None)
                    } else {
                        Ok(Some(Local::from_raw(ret)))
                    }
                }
                Err(e) => Err(e),
            }
        }
    }
}

impl<'ctx> GetReturn<'ctx> for Local<'ctx> {
    unsafe fn get<const STATIC: bool, T: StrongRef>(
        ctx: &'ctx Context,
        this: &T,
        field: Field<STATIC>,
    ) -> Result<Self, Local<'ctx>> {
        unsafe { Option::<Local>::get(ctx, this, field).map(|v| v.expect("unexpected null value returns from java.")) }
    }
}

impl<'a> __sealed::Sealed for &'a Local<'a> {}

impl<'a> SetArg for &'a Local<'a> {
    unsafe fn set<'ctx, const STATIC: bool, T: StrongRef>(
        self,
        ctx: &'ctx Context,
        this: &T,
        field: Field<STATIC>,
    ) -> Result<(), Local<'ctx>> {
        unsafe {
            if STATIC {
                call!(ctx, SetStaticObjectField, *this.as_raw(), field.into_raw(), *self.as_raw())
            } else {
                call!(ctx, SetObjectField, *this.as_raw(), field.into_raw(), *self.as_raw())
            }
        }
    }
}

impl<'a> SetArg for Local<'a> {
    unsafe fn set<'ctx, const STATIC: bool, T: StrongRef>(
        self,
        ctx: &'ctx Context,
        this: &T,
        field: Field<STATIC>,
    ) -> Result<(), Local<'ctx>> {
        unsafe { (&self).set(ctx, this, field) }
    }
}

impl<'a> __sealed::Sealed for Option<&'a Local<'a>> {}

impl<'a> SetArg for Option<&'a Local<'a>> {
    unsafe fn set<'ctx, const STATIC: bool, T: StrongRef>(
        self,
        ctx: &'ctx Context,
        this: &T,
        field: Field<STATIC>,
    ) -> Result<(), Local<'ctx>> {
        unsafe {
            match self {
                None => {
                    if STATIC {
                        call!(ctx, SetStaticObjectField, *this.as_raw(), field.into_raw(), null_mut())
                    } else {
                        call!(ctx, SetObjectField, *this.as_raw(), field.into_raw(), null_mut())
                    }
                }
                Some(l) => l.set(ctx, this, field),
            }
        }
    }
}

impl Context {
    pub unsafe fn get_field<'ctx, const STATIC: bool, T: StrongRef, V: GetReturn<'ctx>>(
        &'ctx self,
        this: &T,
        field: Field<STATIC>,
    ) -> Result<V, Local<'ctx>> {
        unsafe { V::get(self, this, field) }
    }

    pub unsafe fn set_field<const STATIC: bool, T: StrongRef, V: SetArg>(
        &self,
        this: &T,
        field: Field<STATIC>,
        value: V,
    ) -> Result<(), Local<'_>> {
        unsafe { value.set(self, this, field) }
    }
}

#[doc(hidden)]
pub trait PrimitiveArrayElement: Sized + __sealed::Sealed {
    unsafe fn new_array(ctx: &Context, length: i32) -> Result<Local<'_>, Local<'_>>;

    unsafe fn get_region<'ctx, T: StrongRef>(
        ctx: &'ctx Context,
        this: &T,
        offset: i32,
        buf: &mut [Self],
    ) -> Result<(), Local<'ctx>>;

    unsafe fn set_region<'ctx, T: StrongRef>(ctx: &'ctx Context, this: &T, offset: i32, buf: &[Self]) -> Result<(), Local<'ctx>>;

    unsafe fn get_elements<'r, T: StrongRef>(ctx: &'r Context, this: &'r T) -> &'r mut [Self];

    unsafe fn release_elements<T: StrongRef>(ctx: &Context, this: &T, buf: &mut [Self], commit: bool);
}

macro_rules! impl_primitive_array_element {
    ($typ:ty, $new:ident, $get_region:ident, $set_region:ident, $get_elements:ident, $release_elements:ident) => {
        impl PrimitiveArrayElement for $typ {
            unsafe fn new_array(ctx: &Context, length: i32) -> Result<Local<'_>, Local<'_>> {
                unsafe { call!(ctx, $new, length).map(|r| Local::from_raw(r)) }
            }

            unsafe fn get_region<'ctx, T: StrongRef>(
                ctx: &'ctx Context,
                this: &T,
                offset: i32,
                buf: &mut [Self],
            ) -> Result<(), Local<'ctx>> {
                unsafe {
                    call!(
                        ctx,
                        $get_region,
                        *this.as_raw(),
                        offset,
                        buf.len().try_into().unwrap(),
                        buf.as_mut_ptr()
                    )
                }
            }

            unsafe fn set_region<'ctx, T: StrongRef>(
                ctx: &'ctx Context,
                this: &T,
                offset: i32,
                buf: &[Self],
            ) -> Result<(), Local<'ctx>> {
                unsafe {
                    call!(
                        ctx,
                        $set_region,
                        *this.as_raw(),
                        offset,
                        buf.len().try_into().unwrap(),
                        buf.as_ptr()
                    )
                }
            }

            unsafe fn get_elements<'r, T: StrongRef>(ctx: &'r Context, this: &'r T) -> &'r mut [Self] {
                unsafe {
                    let length = call_nothrow!(ctx, GetArrayLength, *this.as_raw());
                    let ptr = call_nothrow!(ctx, $get_elements, *this.as_raw(), null_mut());

                    core::slice::from_raw_parts_mut(ptr, length as _)
                }
            }

            unsafe fn release_elements<T: StrongRef>(ctx: &Context, this: &T, buf: &mut [Self], commit: bool) {
                unsafe {
                    call_nothrow!(
                        ctx,
                        $release_elements,
                        *this.as_raw(),
                        buf.as_mut_ptr(),
                        (if commit { JNI_COMMIT } else { JNI_ABORT }) as i32
                    )
                }
            }
        }
    };
}

impl_primitive_array_element!(
    bool,
    NewBooleanArray,
    GetBooleanArrayRegion,
    SetBooleanArrayRegion,
    GetBooleanArrayElements,
    ReleaseBooleanArrayElements
);
impl_primitive_array_element!(
    i8,
    NewByteArray,
    GetByteArrayRegion,
    SetByteArrayRegion,
    GetByteArrayElements,
    ReleaseByteArrayElements
);
impl_primitive_array_element!(
    u16,
    NewCharArray,
    GetCharArrayRegion,
    SetCharArrayRegion,
    GetCharArrayElements,
    ReleaseCharArrayElements
);
impl_primitive_array_element!(
    i16,
    NewShortArray,
    GetShortArrayRegion,
    SetShortArrayRegion,
    GetShortArrayElements,
    ReleaseShortArrayElements
);
impl_primitive_array_element!(
    i32,
    NewIntArray,
    GetIntArrayRegion,
    SetIntArrayRegion,
    GetIntArrayElements,
    ReleaseIntArrayElements
);
impl_primitive_array_element!(
    i64,
    NewLongArray,
    GetLongArrayRegion,
    SetLongArrayRegion,
    GetLongArrayElements,
    ReleaseLongArrayElements
);
impl_primitive_array_element!(
    f32,
    NewFloatArray,
    GetFloatArrayRegion,
    SetFloatArrayRegion,
    GetFloatArrayElements,
    ReleaseFloatArrayElements
);
impl_primitive_array_element!(
    f64,
    NewDoubleArray,
    GetDoubleArrayRegion,
    SetDoubleArrayRegion,
    GetDoubleArrayElements,
    ReleaseDoubleArrayElements
);

impl Context {
    pub unsafe fn get_array_length<R: StrongRef>(&self, object: &R) -> i32 {
        unsafe { call_nothrow!(self, GetArrayLength, *object.as_raw()) }
    }

    pub unsafe fn new_primitive_array<E: PrimitiveArrayElement>(&self, size: i32) -> Result<Local<'_>, Local<'_>> {
        unsafe { E::new_array(self, size) }
    }

    pub unsafe fn get_primitive_array_region<'ctx, E: PrimitiveArrayElement, T: StrongRef>(
        &'ctx self,
        this: &T,
        offset: i32,
        buf: &mut [E],
    ) -> Result<(), Local<'ctx>> {
        unsafe { E::get_region(self, this, offset, buf) }
    }

    pub unsafe fn set_primitive_array_region<'ctx, E: PrimitiveArrayElement, T: StrongRef>(
        &'ctx self,
        this: &T,
        offset: i32,
        buf: &[E],
    ) -> Result<(), Local<'ctx>> {
        unsafe { E::set_region(self, this, offset, buf) }
    }

    pub unsafe fn get_primitive_array_elements<'r, E: PrimitiveArrayElement, T: StrongRef>(&'r self, this: &'r T) -> &'r mut [E] {
        unsafe { E::get_elements(self, this) }
    }

    pub unsafe fn release_primitive_array_elements<E: PrimitiveArrayElement, T: StrongRef>(
        &self,
        this: &T,
        buf: &mut [E],
        commit: bool,
    ) {
        unsafe { E::release_elements(self, this, buf, commit) }
    }

    pub unsafe fn new_object_array<R1: StrongRef, R2: Ref>(
        &self,
        length: i32,
        class: &R1,
        initial: Option<&R2>,
    ) -> Result<Local<'_>, Local<'_>> {
        unsafe {
            self.ensure_local_capacity(4);

            call!(
                self,
                NewObjectArray,
                length,
                *class.as_raw(),
                initial.map(|r| *r.as_raw()).unwrap_or(null_mut())
            )
            .map(|r| Local::from_raw(r))
        }
    }

    pub unsafe fn get_object_array_element<R: StrongRef>(&self, object: &R, index: i32) -> Result<Option<Local<'_>>, Local<'_>> {
        unsafe {
            self.ensure_local_capacity(4);

            call!(self, GetObjectArrayElement, *object.as_raw(), index)
                .map(|raw| if raw.is_null() { None } else { Some(Local::from_raw(raw)) })
        }
    }

    pub unsafe fn set_object_array_element<R1: StrongRef, R2: Ref>(
        &self,
        object: &R1,
        index: i32,
        value: Option<&R2>,
    ) -> Result<(), Local<'_>> {
        unsafe {
            call!(
                self,
                SetObjectArrayElement,
                *object.as_raw(),
                index,
                value.map(|r| *r.as_raw()).unwrap_or(null_mut())
            )
        }
    }
}

impl Context {
    pub unsafe fn register_natives<const COUNT: usize, R: StrongRef, N: AsRef<CStr>, S: AsRef<CStr>>(
        &self,
        class: &R,
        natives: [(N, S, *const ()); COUNT],
    ) -> Result<(), Local<'_>> {
        unsafe {
            let mut funcs = [MaybeUninit::<JNINativeMethod>::uninit(); COUNT];
            for (index, (name, signature, ptr)) in natives.iter().enumerate() {
                funcs[index] = MaybeUninit::new(JNINativeMethod {
                    name: name.as_ref().as_ptr() as _,
                    signature: signature.as_ref().as_ptr() as _,
                    fnPtr: *ptr as _,
                });
            }

            let n = call!(self, RegisterNatives, *class.as_raw(), funcs.as_ptr() as _, funcs.len() as _)?;
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
