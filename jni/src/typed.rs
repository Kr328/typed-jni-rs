use alloc::{
    borrow::Cow,
    ffi::CString,
    format,
    string::{String, ToString},
};
use core::{
    fmt::{Debug, Display, Formatter},
    marker::PhantomData,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{
    builtin::Throwable,
    context::{CallArg, CallResult, Context, GetReturn, Method, SetArg},
    reference::{Local, Ref, StrongRef},
    resolver,
    sys::_jmethodID,
    AsRaw, FromRaw, Global, IntoRaw, Raw, Weak, WeakRef,
};

mod __sealed {
    pub trait Sealed {}
}

#[derive(Copy, Clone)]
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

pub trait ObjectType: Type {}

pub trait PrimitiveType: Type + __sealed::Sealed {}

impl<'a, T: Type> Type for &'a T {
    const SIGNATURE: Signature = T::SIGNATURE;
}

impl<T: Type> Type for Option<T> {
    const SIGNATURE: Signature = T::SIGNATURE;
}

macro_rules! impl_primitive_type {
    ($typ:ty, $signature:expr) => {
        impl Type for $typ {
            const SIGNATURE: Signature = $signature;
        }

        impl PrimitiveType for $typ {}

        impl __sealed::Sealed for $typ {}
    };
}

impl_primitive_type!((), Signature::Void);
impl_primitive_type!(bool, Signature::Boolean);
impl_primitive_type!(i8, Signature::Byte);
impl_primitive_type!(u16, Signature::Char);
impl_primitive_type!(i16, Signature::Short);
impl_primitive_type!(i32, Signature::Int);
impl_primitive_type!(i64, Signature::Long);
impl_primitive_type!(f32, Signature::Float);
impl_primitive_type!(f64, Signature::Double);

macro_rules! impl_value_for {
    ($typ:ty) => {
        impl Raw for $typ {
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

#[repr(transparent)]
pub struct Class<R: Ref, T: Type> {
    typ: PhantomData<T>,
    reference: R,
}

#[repr(transparent)]
pub struct Object<R: Ref, T: Type> {
    typ: PhantomData<T>,
    reference: R,
}

impl<R: Ref, T: Type> Type for Class<R, T> {
    const SIGNATURE: Signature = Signature::Object("java/lang/Class");
}

impl<R: Ref, T: Type> Type for Object<R, T> {
    const SIGNATURE: Signature = T::SIGNATURE;
}

fn object_to_string<R: StrongRef>(r: &R) -> String {
    Context::with_attached(|ctx| {
        static M_TO_STRING: AtomicPtr<_jmethodID> = AtomicPtr::new(null_mut());
        let m_to_string = M_TO_STRING.load(Ordering::Relaxed);
        let m_to_string = if m_to_string.is_null() {
            match ctx
                .find_class(c"java/lang/Object")
                .and_then(|c| ctx.find_method(&c, c"toString", c"()Ljava/lang/String;"))
            {
                Ok(m) => {
                    M_TO_STRING.store(*m.as_raw(), Ordering::Relaxed);

                    m
                }
                Err(_) => panic!("BROKEN: find java/lang/Object.toString() failed"),
            }
        } else {
            unsafe { Method::<false>::from_raw(m_to_string) }
        };

        unsafe {
            ctx.call_method(r, m_to_string, [])
                .ok()
                .flatten()
                .map(|s| ctx.get_string(&s))
                .unwrap_or("<exception>".to_string())
        }
    })
}

#[derive(Debug)]
pub struct ClassCastException;

impl Display for ClassCastException {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("ClassCastException")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ClassCastException {}

macro_rules! impl_common {
    ($name:ident) => {
        impl<R: Ref, T: Type> Clone for $name<R, T> {
            fn clone(&self) -> Self {
                unsafe { Self::from_raw(self.as_raw().clone()) }
            }
        }

        impl<R: Ref, T: Type> Raw for $name<R, T> {
            type Raw = R;
        }

        impl<R: Ref, T: Type> AsRaw for $name<R, T> {
            fn as_raw(&self) -> &Self::Raw {
                &self.reference
            }
        }

        impl<R: Ref, T: Type> IntoRaw for $name<R, T> {
            fn into_raw(self) -> Self::Raw {
                self.reference
            }
        }

        impl<R: Ref, T: Type> FromRaw for $name<R, T> {
            unsafe fn from_raw(raw: Self::Raw) -> Self {
                Self {
                    reference: raw,
                    typ: PhantomData,
                }
            }
        }

        impl<'a, R: Ref, T: Type> Raw for &'a $name<R, T> {
            type Raw = &'a R;
        }

        impl<'a, R: Ref, T: Type> IntoRaw for &'a $name<R, T> {
            fn into_raw(self) -> Self::Raw {
                &self.reference
            }
        }

        impl<R: StrongRef, T: Type> $name<R, T> {
            pub fn to_global(&self) -> $name<Global, T> {
                unsafe { $name::from_raw(self.as_raw().to_global()) }
            }

            pub fn to_local<'ctx>(&self, ctx: &'ctx Context) -> $name<Local<'ctx>, T> {
                unsafe { $name::from_raw(self.as_raw().to_local(ctx)) }
            }

            pub fn downgrade_weak<'ctx>(&self) -> $name<Weak, T> {
                unsafe { $name::from_raw(self.as_raw().downgrade_weak()) }
            }
        }

        impl<R: WeakRef, T: Type> $name<R, T> {
            pub fn upgrade_global(&self) -> Option<$name<Global, T>> {
                unsafe { self.as_raw().upgrade_global().map(|r| $name::from_raw(r)) }
            }

            pub fn upgrade_local<'ctx>(&self, ctx: &'ctx Context) -> Option<$name<Local<'ctx>, T>> {
                unsafe { self.as_raw().upgrade_local(ctx).map(|r| $name::from_raw(r)) }
            }
        }

        impl<R: StrongRef, T: Type> $name<R, T> {
            pub fn is_instance_of<NR: StrongRef, NT: Type>(&self, ctx: &Context, class: &Class<NR, NT>) -> bool {
                unsafe { ctx.is_instance_of(self.as_raw(), class.as_raw()) }
            }
        }

        impl<R: StrongRef, T: Type> $name<R, T> {
            pub fn cast<'ctx, NR: StrongRef, NT: Type>(
                &self,
                ctx: &'ctx Context,
                class: &Class<NR, NT>,
            ) -> Result<$name<Local<'ctx>, NT>, ClassCastException> {
                if self.is_instance_of(ctx, class) {
                    unsafe { Ok($name::from_raw(self.as_raw().to_local(ctx))) }
                } else {
                    Err(ClassCastException)
                }
            }
        }

        impl<R: StrongRef, T: Type> Debug for $name<R, T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                f.write_str(&object_to_string(&self.reference))
            }
        }

        impl<R: StrongRef, T: Type> Display for $name<R, T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                f.write_str(&object_to_string(&self.reference))
            }
        }
    };
}

impl_common!(Class);
impl_common!(Object);

impl<'ctx, T: Type> Class<Local<'ctx>, T> {
    pub fn find_class(ctx: &'ctx Context) -> Result<Self, Object<Local<'ctx>, Throwable>> {
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

        let class_name = class_name_of(&T::SIGNATURE);

        let class_name = CString::new(class_name.into_owned()).unwrap();

        ctx.find_class(&class_name).map(|r| Self {
            reference: r,
            typ: PhantomData,
        })
    }
}

impl<'ctx, R: StrongRef, T: Type> Class<R, T> {
    pub fn is_assignable_from<SR: StrongRef, ST: Type>(&self, ctx: &Context, superclass: &Class<SR, ST>) -> bool {
        unsafe { ctx.is_assignable_from(self.as_raw(), superclass.as_raw()) }
    }
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

fn call_method<'ctx, 't, 'a, const STATIC: bool, const ARGS: usize, T, R, A>(
    ctx: &'ctx Context,
    this: &T,
    name: &'static str,
    args: A,
) -> Result<R, Object<Local<'ctx>, Throwable>>
where
    T: AsRaw,
    T::Raw: StrongRef,
    R: Type,
    R: FromRaw,
    R::Raw: CallResult<'ctx>,
    A: Args<'a, ARGS>,
{
    let method: Method<STATIC> = if STATIC {
        resolver::find_method::<STATIC, ARGS, _, A, R>(ctx, this.as_raw(), name)?
    } else {
        let class = ctx.get_object_class(this.as_raw());

        resolver::find_method::<STATIC, ARGS, _, A, R>(ctx, &class, name)?
    };

    let raw_args = args.into_raw();

    unsafe { ctx.call_method(this.as_raw(), method, raw_args).map(|v| R::from_raw(v)) }
}

impl<R: StrongRef, T: Type> Object<R, T> {
    pub fn call_method<'ctx, 'a, const ARGS: usize, A, V>(
        &self,
        ctx: &'ctx Context,
        name: &'static str,
        args: A,
    ) -> Result<V, Object<Local<'ctx>, Throwable>>
    where
        V: Type,
        V: FromRaw,
        V::Raw: CallResult<'ctx>,
        A: Args<'a, ARGS>,
    {
        call_method::<false, ARGS, _, _, _>(ctx, self, name, args)
    }
}

impl<R: StrongRef, T: Type> Class<R, T> {
    pub fn call_method<'ctx, 'a, const ARGS: usize, V, A>(
        &self,
        ctx: &'ctx Context,
        name: &'static str,
        args: A,
    ) -> Result<V, Object<Local<'ctx>, Throwable>>
    where
        V: Type,
        V: FromRaw,
        V::Raw: CallResult<'ctx>,
        A: Args<'a, ARGS>,
    {
        call_method::<true, ARGS, _, _, _>(ctx, self, name, args)
    }
}

impl<R: StrongRef, T: Type> Class<R, T> {
    pub fn new_object<'ctx, 'args, const ARGS: usize, A>(
        &self,
        ctx: &'ctx Context,
        args: A,
    ) -> Result<Object<Local<'ctx>, T>, Object<Local<'ctx>, Throwable>>
    where
        A: Args<'args, ARGS>,
        Object<Local<'ctx>, T>: Raw<Raw = Local<'ctx>> + FromRaw,
    {
        let method: Method<false> = resolver::find_method::<false, ARGS, _, A, ()>(ctx, self.as_raw(), "<init>")?;

        let raw_args = args.into_raw();
        unsafe { ctx.new_object(self.as_raw(), method, raw_args).map(|v| Object::from_raw(v)) }
    }
}

fn get_field<'ctx, const STATIC: bool, T, R>(
    ctx: &'ctx Context,
    this: &T,
    name: &'static str,
) -> Result<R, Object<Local<'ctx>, Throwable>>
where
    T: AsRaw,
    T::Raw: StrongRef,
    R: FromRaw + Type,
    R::Raw: GetReturn<'ctx>,
{
    let field = if STATIC {
        resolver::find_field::<STATIC, _, R>(ctx, this.as_raw(), name)?
    } else {
        let class = ctx.get_object_class(this.as_raw());

        resolver::find_field::<STATIC, _, R>(ctx, &class, name)?
    };

    unsafe { Ok(R::from_raw(ctx.get_field(this.as_raw(), field))) }
}

impl<R: StrongRef, T: Type> Object<R, T> {
    pub fn get_field<'ctx, V>(&self, ctx: &'ctx Context, name: &'static str) -> Result<V, Object<Local<'ctx>, Throwable>>
    where
        V: FromRaw + Type,
        V::Raw: GetReturn<'ctx>,
    {
        get_field::<false, _, _>(ctx, self, name)
    }
}

impl<R: StrongRef, T: Type> Class<R, T> {
    pub fn get_field<'ctx, V>(&self, ctx: &'ctx Context, name: &'static str) -> Result<V, Object<Local<'ctx>, Throwable>>
    where
        V: FromRaw + Type,
        V::Raw: GetReturn<'ctx>,
    {
        get_field::<true, _, _>(ctx, self, name)
    }
}

fn set_field<'ctx, const STATIC: bool, T, V>(
    ctx: &'ctx Context,
    this: &T,
    name: &'static str,
    value: V,
) -> Result<(), Object<Local<'ctx>, Throwable>>
where
    T: AsRaw,
    T::Raw: StrongRef,
    V: IntoRaw + Type,
    V::Raw: SetArg,
{
    let field = if STATIC {
        resolver::find_field::<STATIC, _, V>(ctx, this.as_raw(), name)?
    } else {
        let class = ctx.get_object_class(this.as_raw());

        resolver::find_field::<STATIC, _, V>(ctx, &class, name)?
    };

    unsafe { Ok(ctx.set_field(this.as_raw(), field, value.into_raw())) }
}

impl<R: StrongRef, T: Type> Object<R, T> {
    pub fn set_field<'ctx, V>(
        &self,
        ctx: &'ctx Context,
        name: &'static str,
        value: V,
    ) -> Result<(), Object<Local<'ctx>, Throwable>>
    where
        V: IntoRaw + Type,
        V::Raw: SetArg,
    {
        set_field::<false, _, _>(ctx, self, name, value)
    }
}

impl<R: StrongRef, T: Type> Class<R, T> {
    pub fn set_field<'ctx, V>(
        &self,
        ctx: &'ctx Context,
        name: &'static str,
        value: V,
    ) -> Result<(), Object<Local<'ctx>, Throwable>>
    where
        V: IntoRaw + Type,
        V::Raw: SetArg,
    {
        set_field::<true, _, _>(ctx, self, name, value)
    }
}
