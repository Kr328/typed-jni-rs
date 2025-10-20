use crate::{JNIEnv, LocalRef, MethodID, StrongRef, helper::call, sys};

/// Argument for a method call.
#[derive(Debug)]
pub enum Arg<'r> {
    Boolean(bool),
    Byte(i8),
    Char(u16),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Object(Option<&'r dyn StrongRef>),
}

impl<'r> Clone for Arg<'r> {
    fn clone(&self) -> Self {
        match self {
            Arg::Boolean(v) => Self::Boolean(*v),
            Arg::Byte(v) => Self::Byte(*v),
            Arg::Char(v) => Self::Char(*v),
            Arg::Short(v) => Self::Short(*v),
            Arg::Int(v) => Self::Int(*v),
            Arg::Long(v) => Self::Long(*v),
            Arg::Float(v) => Self::Float(*v),
            Arg::Double(v) => Self::Double(*v),
            Arg::Object(v) => Self::Object(*v),
        }
    }
}

impl<'r> Copy for Arg<'r> {}

impl<'r> Arg<'r> {
    #[cfg(debug_assertions)]
    fn enforce_valid_runtime(&self, env: &JNIEnv) {
        match self {
            Self::Object(Some(v)) => v.enforce_valid_runtime(env),
            _ => {}
        }
    }

    fn into_value(self) -> sys::jvalue {
        match self {
            Self::Boolean(v) => sys::jvalue { z: v as _ },
            Self::Byte(v) => sys::jvalue { b: v as _ },
            Self::Char(v) => sys::jvalue { c: v as _ },
            Self::Short(v) => sys::jvalue { s: v as _ },
            Self::Int(v) => sys::jvalue { i: v as _ },
            Self::Long(v) => sys::jvalue { j: v as _ },
            Self::Float(v) => sys::jvalue { f: v as _ },
            Self::Double(v) => sys::jvalue { d: v as _ },
            Self::Object(v) => sys::jvalue {
                l: v.map_or(core::ptr::null_mut(), |v| v.as_raw_ptr() as _),
            },
        }
    }
}

macro_rules! impl_from_primitive {
    ($typ:ty, $variant:ident) => {
        impl<'r> From<$typ> for Arg<'r> {
            fn from(value: $typ) -> Self {
                Self::$variant(value)
            }
        }
    };
}

impl_from_primitive!(bool, Boolean);
impl_from_primitive!(i8, Byte);
impl_from_primitive!(u16, Char);
impl_from_primitive!(i16, Short);
impl_from_primitive!(i32, Int);
impl_from_primitive!(i64, Long);
impl_from_primitive!(f32, Float);
impl_from_primitive!(f64, Double);

impl<'r, R: StrongRef> From<&'r R> for Arg<'r> {
    fn from(value: &'r R) -> Self {
        Self::Object(Some(value))
    }
}

macro_rules! do_call_method {
    ($is_static:expr, $env:expr, $func:ident, $static_func:ident, $obj:expr, $method:expr, $args:expr) => {
        $env.run_catch(|| unsafe {
            if $is_static {
                call!(
                    $env.as_raw_ptr(),
                    $static_func,
                    $obj.as_raw_ptr(),
                    $method.as_raw_ptr(),
                    $args.as_ptr()
                )
            } else {
                call!(
                    $env.as_raw_ptr(),
                    $func,
                    $obj.as_raw_ptr(),
                    $method.as_raw_ptr(),
                    $args.as_ptr()
                )
            }
        })
    };
}

macro_rules! define_call_func {
    ($name:ident, $name_variadic:ident, $ret:ty, $func:ident, $static_func:ident, $remap_ret:path, $doc:literal) => {
        impl<'vm> JNIEnv<'vm> {
            #[doc = $doc]
            pub unsafe fn $name<const STATIC: bool, const N_ARGS: usize, R: StrongRef>(
                &self,
                obj: &R,
                method: MethodID<STATIC>,
                args: [Arg<'_>; N_ARGS],
            ) -> Result<$ret, LocalRef<'_>> {
                #[cfg(debug_assertions)]
                obj.enforce_valid_runtime(self);

                let args = args.map(|arg| {
                    #[cfg(debug_assertions)]
                    arg.enforce_valid_runtime(self);

                    arg.into_value()
                });

                let ret = do_call_method!(STATIC, self, $func, $static_func, obj, method, args)?;

                Ok($remap_ret(self, ret))
            }

            #[cfg(feature = "alloc")]
            #[doc = $doc]
            pub unsafe fn $name_variadic<'a, const STATIC: bool, R: StrongRef, Args: IntoIterator<Item = Arg<'a>>>(
                &self,
                obj: &R,
                method: MethodID<STATIC>,
                args: Args,
            ) -> Result<$ret, LocalRef<'_>> {
                #[cfg(debug_assertions)]
                obj.enforce_valid_runtime(self);

                let args = args
                    .into_iter()
                    .map(|arg| {
                        #[cfg(debug_assertions)]
                        arg.enforce_valid_runtime(self);

                        arg.into_value()
                    })
                    .collect::<alloc::vec::Vec<_>>();

                let ret = do_call_method!(STATIC, self, $func, $static_func, obj, method, args)?;

                Ok($remap_ret(self, ret))
            }
        }
    };
    ($name:ident, $name_variadic:ident, $ret:ty, $static_func:ident, $func:ident, $doc:literal) => {
        const _: () = {
            #[inline(always)]
            fn remap_as_is<T>(_: &JNIEnv, v: T) -> T {
                v
            }

            define_call_func!($name, $name_variadic, $ret, $static_func, $func, remap_as_is, $doc);
        };
    };
}

define_call_func!(
    call_boolean_method,
    call_boolean_method_variadic,
    bool,
    CallBooleanMethodA,
    CallStaticBooleanMethodA,
    "Call a boolean method.\n\n# Safety\n\n- `method` must be a valid method of `obj`.\n- `args` must match the method signature."
);

define_call_func!(
    call_byte_method,
    call_byte_method_variadic,
    i8,
    CallByteMethodA,
    CallStaticByteMethodA,
    "Call a byte method.\n\n# Safety\n\n- `method` must be a valid method of `obj`.\n- `args` must match the method signature."
);

define_call_func!(
    call_char_method,
    call_char_method_variadic,
    u16,
    CallCharMethodA,
    CallStaticCharMethodA,
    "Call a char method.\n\n# Safety\n\n- `method` must be a valid method of `obj`.\n- `args` must match the method signature."
);

define_call_func!(
    call_short_method,
    call_short_method_variadic,
    i16,
    CallShortMethodA,
    CallStaticShortMethodA,
    "Call a short method.\n\n# Safety\n\n- `method` must be a valid method of `obj`.\n- `args` must match the method signature."
);

define_call_func!(
    call_int_method,
    call_int_method_variadic,
    i32,
    CallIntMethodA,
    CallStaticIntMethodA,
    "Call an int method.\n\n# Safety\n\n- `method` must be a valid method of `obj`.\n- `args` must match the method signature."
);

define_call_func!(
    call_long_method,
    call_long_method_variadic,
    i64,
    CallLongMethodA,
    CallStaticLongMethodA,
    "Call a long method.\n\n# Safety\n\n- `method` must be a valid method of `obj`.\n- `args` must match the method signature."
);

define_call_func!(
    call_float_method,
    call_float_method_variadic,
    f32,
    CallFloatMethodA,
    CallStaticFloatMethodA,
    "Call a float method.\n\n# Safety\n\n- `method` must be a valid method of `obj`.\n- `args` must match the method signature."
);

define_call_func!(
    call_double_method,
    call_double_method_variadic,
    f64,
    CallDoubleMethodA,
    CallStaticDoubleMethodA,
    "Call a double method.\n\n# Safety\n\n- `method` must be a valid method of `obj`.\n- `args` must match the method signature."
);

define_call_func!(
    call_void_method,
    call_void_method_variadic,
    (),
    CallVoidMethodA,
    CallStaticVoidMethodA,
    "Call a void method.\n\n# Safety\n\n- `method` must be a valid method of `obj`.\n- `args` must match the method signature."
);

fn remap_as_local_ref_or_null<'env>(env: &'env JNIEnv, v: sys::jobject) -> Option<LocalRef<'env>> {
    if !v.is_null() {
        Some(unsafe { LocalRef::from_raw(env, v) })
    } else {
        None
    }
}

define_call_func!(
    call_object_method,
    call_object_method_variadic,
    Option<LocalRef<'_>>,
    CallObjectMethodA,
    CallStaticObjectMethodA,
    remap_as_local_ref_or_null,
    "Call an object method.\n\n# Safety\n\n- `method` must be a valid method of `obj`.\n- `args` must match the method signature."
);

impl<'vm> JNIEnv<'vm> {
    /// Create a new object instance.
    ///
    /// # Safety
    ///
    /// - `cls` must be a valid class.
    /// - `method` must be a valid constructor of `cls`.
    /// - `args` must match the method signature.
    pub unsafe fn new_object<const N_ARGS: usize, R: StrongRef>(
        &self,
        cls: &R,
        method: MethodID<false>,
        args: [Arg<'_>; N_ARGS],
    ) -> Result<LocalRef<'_>, LocalRef<'_>> {
        #[cfg(debug_assertions)]
        cls.enforce_valid_runtime(self);

        #[cfg(debug_assertions)]
        args.iter().for_each(|arg| arg.enforce_valid_runtime(self));

        let ret = self.run_catch(|| unsafe {
            let args = args.map(|arg| arg.into_value());

            call!(
                self.as_raw_ptr(),
                NewObjectA,
                cls.as_raw_ptr(),
                method.as_raw_ptr(),
                args.as_ptr()
            )
        })?;

        unsafe { Ok(LocalRef::from_raw(self, ret)) }
    }

    /// Create a new object instance with variadic arguments.
    ///
    /// # Safety
    ///
    /// - `cls` must be a valid class.
    /// - `method` must be a valid constructor of `cls`.
    /// - `args` must match the method signature.
    #[cfg(feature = "alloc")]
    pub unsafe fn new_object_variadic<'a, R: StrongRef, Args: IntoIterator<Item = Arg<'a>>>(
        &self,
        cls: &R,
        method: MethodID<false>,
        args: Args,
    ) -> Result<LocalRef<'_>, LocalRef<'_>> {
        #[cfg(debug_assertions)]
        cls.enforce_valid_runtime(self);

        let args = args
            .into_iter()
            .map(|arg| {
                #[cfg(debug_assertions)]
                arg.enforce_valid_runtime(self);

                arg.into_value()
            })
            .collect::<alloc::vec::Vec<_>>();

        let ret = self.run_catch(|| unsafe {
            call!(
                self.as_raw_ptr(),
                NewObjectA,
                cls.as_raw_ptr(),
                method.as_raw_ptr(),
                args.as_ptr()
            )
        })?;

        unsafe { Ok(LocalRef::from_raw(self, ret)) }
    }
}
