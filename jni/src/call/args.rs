use typed_jni_core::{Arg, JNIEnv, MethodID, StrongRef};

use crate::{LocalObject, Null, Object, ObjectType, Signature, Type, builtin::JavaThrowable, call::target::Target};

/// Converts a value to a JNI call argument.
///
/// Supported Types:
/// - Primitives: `bool`, `i8`, `u16`, `i16`, `i32`, `i64`, `f32`, `f64`
/// - Any [Object] with [`StrongRef`]: `Object<impl StrongRef, Type>`, `Option<Object<impl StrongRef, Type>>`
/// - Any reference to [Object] with [`StrongRef`]: `&Object<impl StrongRef, Type>`, `Option<&Object<impl StrongRef, Type>>`
pub trait ToArg {
    fn to_arg(&self) -> Arg<'_>;
}

macro_rules! impl_to_arg_primitive {
    ($t:ty, $variant:ident) => {
        impl ToArg for $t {
            fn to_arg(&self) -> Arg<'_> {
                Arg::$variant(*self)
            }
        }
    };
}

impl_to_arg_primitive!(bool, Boolean);
impl_to_arg_primitive!(i8, Byte);
impl_to_arg_primitive!(u16, Char);
impl_to_arg_primitive!(i16, Short);
impl_to_arg_primitive!(i32, Int);
impl_to_arg_primitive!(i64, Long);
impl_to_arg_primitive!(f32, Float);
impl_to_arg_primitive!(f64, Double);

impl<R: StrongRef, T: ObjectType> ToArg for Object<R, T> {
    fn to_arg(&self) -> Arg<'_> {
        Arg::Object(Some(&**self))
    }
}

impl<R: StrongRef, T: ObjectType> ToArg for Option<Object<R, T>> {
    fn to_arg(&self) -> Arg<'_> {
        match self {
            Some(obj) => Arg::Object(Some(&**obj)),
            None => Arg::Object(None),
        }
    }
}

impl<R: StrongRef, T: ObjectType> ToArg for &Object<R, T> {
    fn to_arg(&self) -> Arg<'_> {
        Arg::Object(Some(&***self))
    }
}

impl<R: StrongRef, T: ObjectType> ToArg for Option<&Object<R, T>> {
    fn to_arg(&self) -> Arg<'_> {
        match self {
            Some(obj) => Arg::Object(Some(&***obj)),
            None => Arg::Object(None),
        }
    }
}

impl<T: ObjectType> ToArg for Null<T> {
    fn to_arg(&self) -> Arg<'_> {
        Arg::Object(None)
    }
}

/// Args to be applied to a JNI call.
///
/// # Safety
///
/// The implementer must ensure that the `signature` matches the signature of the arguments.
pub unsafe trait Args: Sized {
    fn signature(&self) -> impl IntoIterator<Item = Signature> + Clone + '_;

    /// Apply the arguments to a JNI call.
    ///
    /// # Safety
    ///
    /// The implementer must ensure that the `signature` matches the signature of the arguments.
    unsafe fn apply_on<'env, const STATIC: bool, T, R>(
        self,
        env: &'env JNIEnv,
        this: &T,
        method: MethodID<STATIC>,
    ) -> Result<R, LocalObject<'env, JavaThrowable>>
    where
        T: StrongRef,
        R: Target<'env>;
}

unsafe impl Args for () {
    fn signature(&self) -> impl IntoIterator<Item = Signature> + Clone + '_ {
        []
    }

    unsafe fn apply_on<'env, const STATIC: bool, T, R>(
        self,
        env: &'env JNIEnv,
        this: &T,
        method: MethodID<STATIC>,
    ) -> Result<R, LocalObject<'env, JavaThrowable>>
    where
        T: StrongRef,
        R: Target<'env>,
    {
        unsafe { R::call(env, this, method, []) }
    }
}

macro_rules! impl_fixed_args {
    ($($n:ident),*) => {
        unsafe impl<$($n: ToArg + Type),*> Args for ($($n,)*) {
            fn signature(&self) -> impl IntoIterator<Item = Signature> + Clone + '_ {
                [$($n::SIGNATURE,)*]
            }

            unsafe fn apply_on<'env, const STATIC: bool, T, R>(
                self,
                env: &'env JNIEnv,
                this: &T,
                method: MethodID<STATIC>,
            ) -> Result<R, LocalObject<'env, JavaThrowable>>
            where
                T: StrongRef,
                R: Target<'env>,
            {
                unsafe {
                    #[allow(non_snake_case)]
                    let ($($n,)*) = self;

                    R::call(env, this, method, [$($n.to_arg(),)*])
                }
            }
        }
    };
}

impl_fixed_args!(A1);
impl_fixed_args!(A1, A2);
impl_fixed_args!(A1, A2, A3);
impl_fixed_args!(A1, A2, A3, A4);
impl_fixed_args!(A1, A2, A3, A4, A5, A6);
impl_fixed_args!(A1, A2, A3, A4, A5, A6, A7);
impl_fixed_args!(A1, A2, A3, A4, A5, A6, A7, A8);
impl_fixed_args!(A1, A2, A3, A4, A5, A6, A7, A8, A9);
impl_fixed_args!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10);
impl_fixed_args!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11);
impl_fixed_args!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12);
impl_fixed_args!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13);
impl_fixed_args!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14);
impl_fixed_args!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15);
impl_fixed_args!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16);
impl_fixed_args!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17);
impl_fixed_args!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18
);
impl_fixed_args!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19
);
impl_fixed_args!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20
);
impl_fixed_args!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21
);
impl_fixed_args!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22
);
impl_fixed_args!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23
);
impl_fixed_args!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23, A24
);
impl_fixed_args!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23, A24, A25
);
impl_fixed_args!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23, A24, A25, A26
);
impl_fixed_args!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23, A24, A25, A26, A27
);
impl_fixed_args!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23, A24, A25, A26, A27,
    A28
);
impl_fixed_args!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23, A24, A25, A26, A27,
    A28, A29
);
impl_fixed_args!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23, A24, A25, A26, A27,
    A28, A29, A30
);
impl_fixed_args!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23, A24, A25, A26, A27,
    A28, A29, A30, A31
);
impl_fixed_args!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23, A24, A25, A26, A27,
    A28, A29, A30, A31, A32
);

/// A dynamic argument to be applied to a JNI call.
pub trait DynArg: ToArg {
    fn signature(&self) -> Signature;
}

impl<T: Type + ToArg> DynArg for T {
    fn signature(&self) -> Signature {
        T::SIGNATURE
    }
}

unsafe impl Args for &[&dyn DynArg] {
    fn signature(&self) -> impl IntoIterator<Item = Signature> + Clone + '_ {
        self.iter().map(|arg| arg.signature())
    }

    unsafe fn apply_on<'env, const STATIC: bool, T, R>(
        self,
        env: &'env JNIEnv,
        this: &T,
        method: MethodID<STATIC>,
    ) -> Result<R, LocalObject<'env, JavaThrowable>>
    where
        T: StrongRef,
        R: Target<'env>,
    {
        unsafe { R::call_variadic(env, this, method, self.iter().map(|arg| arg.to_arg())) }
    }
}

unsafe impl<const N: usize> Args for [&dyn DynArg; N] {
    fn signature(&self) -> impl IntoIterator<Item = Signature> + Clone + '_ {
        self.map(|arg| arg.signature())
    }

    unsafe fn apply_on<'env, const STATIC: bool, T, R>(
        self,
        env: &'env JNIEnv,
        this: &T,
        method: MethodID<STATIC>,
    ) -> Result<R, LocalObject<'env, JavaThrowable>>
    where
        T: StrongRef,
        R: Target<'env>,
    {
        unsafe { R::call(env, this, method, self.map(|arg| arg.to_arg())) }
    }
}

unsafe impl<const N: usize> Args for &[&dyn DynArg; N] {
    fn signature(&self) -> impl IntoIterator<Item = Signature> + Clone + '_ {
        (*self).signature()
    }

    unsafe fn apply_on<'env, const STATIC: bool, T, R>(
        self,
        env: &'env JNIEnv,
        this: &T,
        method: MethodID<STATIC>,
    ) -> Result<R, LocalObject<'env, JavaThrowable>>
    where
        T: StrongRef,
        R: Target<'env>,
    {
        unsafe { (*self).apply_on(env, this, method) }
    }
}
