use alloc::{borrow::Cow, ffi::CString, format};
use core::fmt::{Display, Formatter};

use crate::{
    context::{CallArg, CallResult, Context, GetReturn, SetArg},
    reference::{Global, Local, Ref, StrongRef, Weak, WeakRef},
    resolver,
    throwable::JavaThrowable,
    Method,
};

#[derive(Clone)]
pub enum Signature {
    Void,
    Boolean,
    Byte,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Object(&'static str),
    Array(&'static Signature),
}

impl Display for Signature {
    fn fmt(&self, f: &mut Formatter<'_>) -> alloc::fmt::Result {
        match self {
            Signature::Void => f.write_str("V"),
            Signature::Boolean => f.write_str("Z"),
            Signature::Byte => f.write_str("B"),
            Signature::Char => f.write_str("C"),
            Signature::Short => f.write_str("S"),
            Signature::Int => f.write_str("I"),
            Signature::Long => f.write_str("J"),
            Signature::Float => f.write_str("F"),
            Signature::Double => f.write_str("D"),
            Signature::Object(name) => f.write_fmt(format_args!("L{};", name)),
            Signature::Array(inner) => f.write_fmt(format_args!("[{}", inner)),
        }
    }
}

pub trait Type: Sized {
    const SIGNATURE: Signature;
}

macro_rules! impl_type {
    ($typ:ty, $signature:expr) => {
        impl Type for $typ {
            const SIGNATURE: Signature = $signature;
        }
    };
}

impl_type!((), Signature::Void);
impl_type!(bool, Signature::Boolean);
impl_type!(i8, Signature::Byte);
impl_type!(u16, Signature::Char);
impl_type!(i16, Signature::Short);
impl_type!(i32, Signature::Int);
impl_type!(i64, Signature::Long);
impl_type!(f32, Signature::Float);
impl_type!(f64, Signature::Double);

impl<'a, T: Type> Type for &'a T {
    const SIGNATURE: Signature = T::SIGNATURE;
}

impl<T: Type> Type for Option<T> {
    const SIGNATURE: Signature = T::SIGNATURE;
}

pub trait Value {
    type Raw;
}

impl<V: Value> Value for Option<V> {
    type Raw = Option<V::Raw>;
}

pub trait IntoRaw: Value {
    fn into_raw(self) -> Self::Raw;
}

impl<V: IntoRaw> IntoRaw for Option<V> {
    fn into_raw(self) -> Self::Raw {
        self.map(|v| v.into_raw())
    }
}

pub trait FromRaw: Value {
    unsafe fn from_raw(raw: Self::Raw) -> Self;
}

impl<V: FromRaw> FromRaw for Option<V> {
    unsafe fn from_raw(raw: Self::Raw) -> Self {
        raw.map(|r| V::from_raw(r))
    }
}

macro_rules! impl_value_for {
    ($typ:ty) => {
        impl Value for $typ {
            type Raw = $typ;
        }

        impl IntoRaw for $typ {
            fn into_raw(self) -> Self::Raw {
                self
            }
        }

        impl FromRaw for $typ {
            unsafe fn from_raw(raw: Self::Raw) -> Self {
                raw
            }
        }
    };
}

impl_value_for!(());
impl_value_for!(bool);
impl_value_for!(i8);
impl_value_for!(u16);
impl_value_for!(i16);
impl_value_for!(i32);
impl_value_for!(i64);
impl_value_for!(f32);
impl_value_for!(f64);

pub trait Class {
    type Object<R: Ref>;
}

impl<'a, T: Class> Class for &'a T {
    type Object<R: Ref> = T::Object<R>;
}

pub trait Object {
    type Class<R: Ref>;
}

impl<'a, T: Object> Object for &'a T {
    type Class<R: Ref> = T::Class<R>;
}

pub trait Args<'a, const N: usize>: 'a {
    fn signatures() -> [Signature; N];

    fn into_raw(self) -> [CallArg<'a>; N];
}

macro_rules! impl_args {
    ($n:literal, $($args:ident),*) => {
        impl<'a, $($args: Type + IntoRaw + 'a),*> Args<'a, $n> for ($($args),*) where $($args::Raw: Into<CallArg<'a>>),* {
            fn signatures() -> [Signature; $n] {
                [$($args::SIGNATURE),*]
            }

            #[allow(non_snake_case)]
            fn into_raw(self) -> [CallArg<'a>;$n] {
                let ($($args),*) = self;

                [$($args.into_raw().into()),*]
            }
        }
    };
}

impl_args!(2, A1, A2);
impl_args!(3, A1, A2, A3);
impl_args!(4, A1, A2, A3, A4);
impl_args!(5, A1, A2, A3, A4, A5);
impl_args!(6, A1, A2, A3, A4, A5, A6);
impl_args!(7, A1, A2, A3, A4, A5, A6, A7);
impl_args!(8, A1, A2, A3, A4, A5, A6, A7, A8);
impl_args!(9, A1, A2, A3, A4, A5, A6, A7, A8, A9);
impl_args!(10, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10);
impl_args!(11, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11);
impl_args!(12, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12);
impl_args!(13, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13);
impl_args!(14, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14);
impl_args!(15, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15);
impl_args!(16, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16);
impl_args!(17, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17);
impl_args!(18, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18);
impl_args!(19, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19);
impl_args!(20, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20);
impl_args!(21, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21);
impl_args!(22, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22);
impl_args!(23, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23);
impl_args!(24, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23, A24);

impl<'a> Args<'a, 0> for () {
    fn signatures() -> [Signature; 0] {
        []
    }

    fn into_raw(self) -> [CallArg<'a>; 0] {
        []
    }
}

impl<'a, A1: Type + IntoRaw + 'a> Args<'a, 1> for A1
where
    A1::Raw: Into<CallArg<'a>>,
{
    fn signatures() -> [Signature; 1] {
        [A1::SIGNATURE]
    }

    fn into_raw(self) -> [CallArg<'a>; 1] {
        [self.into_raw().into()]
    }
}

pub trait This<const STATIC: bool> {
    type Ref: StrongRef;

    fn as_ref(&self) -> &Self::Ref;
}

impl<'a, const STATIC: bool, T: This<STATIC>> This<STATIC> for &'a T {
    type Ref = T::Ref;

    fn as_ref(&self) -> &Self::Ref {
        (*self).as_ref()
    }
}

pub trait FindClass<'ctx>: Type + This<true> + Class + Value<Raw = Local<'ctx>> + FromRaw
where
    Self::Object<Local<'ctx>>: Type,
{
    fn find_class(ctx: &'ctx Context) -> Result<Self, JavaThrowable<false, Local<'ctx>>>;
}

impl<'ctx, V: Type + This<true> + Class + Value<Raw = Local<'ctx>> + FromRaw> FindClass<'ctx> for V
where
    Self::Object<Local<'ctx>>: Type,
{
    fn find_class(ctx: &'ctx Context) -> Result<Self, JavaThrowable<false, Local<'ctx>>> {
        fn class_name_of(signature: &Signature) -> Cow<'static, str> {
            match signature {
                Signature::Void => Cow::Borrowed("java/lang/Void"),
                Signature::Boolean => Cow::Borrowed("java/lang/Boolean"),
                Signature::Byte => Cow::Borrowed("java/lang/Byte"),
                Signature::Char => Cow::Borrowed("java/lang/Char"),
                Signature::Short => Cow::Borrowed("java/lang/Short"),
                Signature::Int => Cow::Borrowed("java/lang/Int"),
                Signature::Long => Cow::Borrowed("java/lang/Long"),
                Signature::Float => Cow::Borrowed("java/lang/Float"),
                Signature::Double => Cow::Borrowed("java/lang/Double"),
                Signature::Object(cls) => Cow::Borrowed(cls),
                Signature::Array(Signature::Array(s)) => Cow::Owned(format!("[{}", class_name_of(&Signature::Array(s)))),
                Signature::Array(s) => Cow::Owned(format!("[L{};", class_name_of(s))),
            }
        }

        let class_name = class_name_of(&Self::Object::<Local<'ctx>>::SIGNATURE);

        let class_name = CString::new(class_name.into_owned()).unwrap();

        ctx.find_class(&class_name).map(|r| unsafe { Self::from_raw(r) })
    }
}

pub trait CallMethod<const STATIC: bool>: This<STATIC> + Type {
    fn call_method<'ctx, 't, 'args, const ARGS: usize, R: FromRaw + Type, A: Args<'args, ARGS>>(
        &self,
        ctx: &'ctx Context,
        name: &'static str,
        args: A,
    ) -> Result<R, JavaThrowable<false, Local<'ctx>>>
    where
        R::Raw: CallResult<'ctx>;
}

impl<const STATIC: bool, T: This<STATIC> + Type> CallMethod<STATIC> for T {
    fn call_method<'ctx, 't, 'args, const ARGS: usize, R: FromRaw + Type, A: Args<'args, ARGS>>(
        &self,
        ctx: &'ctx Context,
        name: &'static str,
        args: A,
    ) -> Result<R, JavaThrowable<false, Local<'ctx>>>
    where
        R::Raw: CallResult<'ctx>,
    {
        let method: Method<STATIC> = if STATIC {
            resolver::find_method::<STATIC, ARGS, _, A, R>(ctx, self.as_ref(), name)?
        } else {
            let class = ctx.get_object_class(self.as_ref());

            resolver::find_method::<STATIC, ARGS, _, A, R>(ctx, &class, name)?
        };

        let raw_args = args.into_raw();

        unsafe { ctx.call_method(self.as_ref(), method, raw_args).map(|v| R::from_raw(v)) }
    }
}

pub trait NewObject: Class + This<true> + Type {
    fn new_object<'ctx, 'args, const ARGS: usize, A: Args<'args, ARGS>>(
        &self,
        ctx: &'ctx Context,
        args: A,
    ) -> Result<Self::Object<Local<'ctx>>, JavaThrowable<false, Local<'ctx>>>
    where
        Self::Object<Local<'ctx>>: Value<Raw = Local<'ctx>> + FromRaw;
}

impl<T: Class + This<true> + Type> NewObject for T {
    fn new_object<'ctx, 'args, const ARGS: usize, A: Args<'args, ARGS>>(
        &self,
        ctx: &'ctx Context,
        args: A,
    ) -> Result<Self::Object<Local<'ctx>>, JavaThrowable<false, Local<'ctx>>>
    where
        Self::Object<Local<'ctx>>: Value<Raw = Local<'ctx>> + FromRaw,
    {
        let method: Method<false> = resolver::find_method::<false, ARGS, _, A, ()>(ctx, self.as_ref(), "<init>")?;

        let raw_args = args.into_raw();
        unsafe {
            ctx.new_object(self.as_ref(), method, raw_args)
                .map(|v| Self::Object::from_raw(v))
        }
    }
}

pub trait GetField<const STATIC: bool>: This<STATIC> {
    fn get_field<'ctx, R: FromRaw + Type>(
        &self,
        ctx: &'ctx Context,
        name: &'static str,
    ) -> Result<R, JavaThrowable<false, Local<'ctx>>>
    where
        R::Raw: GetReturn<'ctx>;
}

impl<const STATIC: bool, T: This<STATIC>> GetField<STATIC> for T {
    fn get_field<'ctx, R: FromRaw + Type>(
        &self,
        ctx: &'ctx Context,
        name: &'static str,
    ) -> Result<R, JavaThrowable<false, Local<'ctx>>>
    where
        R::Raw: GetReturn<'ctx>,
    {
        let field = if STATIC {
            resolver::find_field::<STATIC, _, R>(ctx, self.as_ref(), name)?
        } else {
            let class = ctx.get_object_class(self.as_ref());

            resolver::find_field::<STATIC, _, R>(ctx, &class, name)?
        };

        unsafe { Ok(R::from_raw(ctx.get_field(self.as_ref(), field))) }
    }
}

pub trait SetField<const STATIC: bool>: This<STATIC> {
    fn set_field<'ctx, V: IntoRaw + Type>(
        &self,
        ctx: &'ctx Context,
        name: &'static str,
        value: V,
    ) -> Result<(), JavaThrowable<false, Local<'ctx>>>
    where
        V::Raw: SetArg;
}

impl<const STATIC: bool, T: This<STATIC>> SetField<STATIC> for T {
    fn set_field<'ctx, V: IntoRaw + Type>(
        &self,
        ctx: &'ctx Context,
        name: &'static str,
        value: V,
    ) -> Result<(), JavaThrowable<false, Local<'ctx>>>
    where
        V::Raw: SetArg,
    {
        let field = if STATIC {
            resolver::find_field::<STATIC, _, V>(ctx, self.as_ref(), name)?
        } else {
            let class = ctx.get_object_class(self.as_ref());

            resolver::find_field::<STATIC, _, V>(ctx, &class, name)?
        };

        unsafe { Ok(ctx.set_field(self.as_ref(), field, value.into_raw())) }
    }
}

pub trait RefConvert {
    type Ref: Ref;
    type Converted<R: Ref>;

    fn as_raw_ref(&self) -> &Self::Ref;
    unsafe fn from_raw_ref<R: Ref>(r: R) -> Self::Converted<R>;
}

pub trait ToRef: RefConvert {
    fn to_global(&self) -> Self::Converted<Global>
    where
        Self::Ref: StrongRef,
    {
        unsafe { Self::from_raw_ref(self.as_raw_ref().to_global()) }
    }

    fn to_local<'ctx>(&self, ctx: &'ctx Context) -> Self::Converted<Local<'ctx>>
    where
        Self::Ref: StrongRef,
    {
        unsafe { Self::from_raw_ref(self.as_raw_ref().to_local(ctx)) }
    }

    fn downgrade_weak<'ctx>(&self) -> Self::Converted<Weak>
    where
        Self::Ref: StrongRef,
    {
        unsafe { Self::from_raw_ref(self.as_raw_ref().downgrade_weak()) }
    }

    fn upgrade_global(&self) -> Option<Self::Converted<Global>>
    where
        Self::Ref: WeakRef,
    {
        unsafe { self.as_raw_ref().upgrade_global().map(|r| Self::from_raw_ref(r)) }
    }

    fn upgrade_local<'ctx>(&self, ctx: &'ctx Context) -> Option<Self::Converted<Local<'ctx>>>
    where
        Self::Ref: WeakRef,
    {
        unsafe { self.as_raw_ref().upgrade_local(ctx).map(|r| Self::from_raw_ref(r)) }
    }
}

impl<V: RefConvert> ToRef for V {}

pub trait UnsafeCast<R>: Value<Raw = R> + IntoRaw {
    unsafe fn unsafe_cast<V: Value<Raw = R> + FromRaw>(self) -> V;
}

impl<R, T: Value<Raw = R> + IntoRaw> UnsafeCast<R> for T {
    unsafe fn unsafe_cast<V: Value<Raw = R> + FromRaw>(self) -> V {
        V::from_raw(self.into_raw())
    }
}
