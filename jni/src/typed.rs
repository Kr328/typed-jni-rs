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
    AsRaw, CallArgs, FromRaw, Global, IntoRaw, Raw, Trampoline, Weak, WeakRef,
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
pub struct Class<T: ObjectType, R: Ref> {
    typ: PhantomData<T>,
    reference: R,
}

#[repr(transparent)]
pub struct Object<T: ObjectType, R: Ref> {
    typ: PhantomData<T>,
    reference: R,
}

impl<T: ObjectType, R: Ref> Type for Class<T, R> {
    const SIGNATURE: Signature = Signature::Object("java/lang/Class");
}

impl<T: ObjectType, R: Ref> Type for Object<T, R> {
    const SIGNATURE: Signature = T::SIGNATURE;
}

pub type LocalObject<'ctx, T> = Object<T, Local<'ctx>>;
pub type TrampolineObject<'ctx, T> = Object<T, Trampoline<'ctx>>;
pub type GlobalObject<T> = Object<T, Global>;
pub type WeakObject<T> = Object<T, Weak>;

pub type LocalClass<'ctx, T> = Class<T, Local<'ctx>>;
pub type TrampolineClass<'ctx, T> = Class<T, Trampoline<'ctx>>;
pub type GlobalClass<T> = Class<T, Global>;
pub type WeakClass<T> = Class<T, Weak>;

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

fn ref_equal<R1: Ref, R2: Ref>(r1: &R1, r2: &R2) -> bool {
    Context::with_attached(|ctx| ctx.is_same_object(Some(r1), Some(r2)))
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
        impl<T: ObjectType, R: Ref + Clone> Clone for $name<T, R> {
            fn clone(&self) -> Self {
                unsafe { Self::from_raw(self.as_raw().clone()) }
            }
        }

        impl<T: ObjectType, R: Ref> Raw for $name<T, R> {
            type Raw = R;
        }

        impl<T: ObjectType, R: Ref> AsRaw for $name<T, R> {
            fn as_raw(&self) -> &Self::Raw {
                &self.reference
            }
        }

        impl<T: ObjectType, R: Ref> IntoRaw for $name<T, R> {
            fn into_raw(self) -> Self::Raw {
                self.reference
            }
        }

        impl<T: ObjectType, R: Ref> FromRaw for $name<T, R> {
            unsafe fn from_raw(raw: Self::Raw) -> Self {
                Self {
                    reference: raw,
                    typ: PhantomData,
                }
            }
        }

        impl<'a, T: ObjectType, R: Ref> Raw for &'a $name<T, R> {
            type Raw = &'a R;
        }

        impl<'a, T: ObjectType, R: Ref> IntoRaw for &'a $name<T, R> {
            fn into_raw(self) -> Self::Raw {
                &self.reference
            }
        }

        impl<T: ObjectType, R: StrongRef> $name<T, R> {
            pub fn to_global(&self) -> $name<T, Global> {
                unsafe { $name::from_raw(self.as_raw().to_global()) }
            }

            pub fn to_local<'ctx>(&self, ctx: &'ctx Context) -> $name<T, Local<'ctx>> {
                unsafe { $name::from_raw(self.as_raw().to_local(ctx)) }
            }

            pub fn downgrade_weak(&self) -> $name<T, Weak> {
                unsafe { $name::from_raw(self.as_raw().downgrade_weak()) }
            }
        }

        impl<T: ObjectType, R: WeakRef> $name<T, R> {
            pub fn upgrade_global(&self) -> Option<$name<T, Global>> {
                unsafe { self.as_raw().upgrade_global().map(|r| $name::from_raw(r)) }
            }

            pub fn upgrade_local<'ctx>(&self, ctx: &'ctx Context) -> Option<$name<T, Local<'ctx>>> {
                unsafe { self.as_raw().upgrade_local(ctx).map(|r| $name::from_raw(r)) }
            }
        }

        impl<T: ObjectType, R: StrongRef> $name<T, R> {
            pub fn is_instance_of<NT: ObjectType, NR: StrongRef>(&self, ctx: &Context, class: &Class<NT, NR>) -> bool {
                unsafe { ctx.is_instance_of(self.as_raw(), class.as_raw()) }
            }
        }

        impl<T: ObjectType, R: StrongRef> $name<T, R> {
            pub unsafe fn cast<'ctx, NT: ObjectType, NR: StrongRef>(
                &self,
                ctx: &'ctx Context,
                class: &Class<NT, NR>,
            ) -> Result<$name<NT, Local<'ctx>>, ClassCastException> {
                if self.is_instance_of(ctx, class) {
                    unsafe { Ok($name::from_raw(self.as_raw().to_local(ctx))) }
                } else {
                    Err(ClassCastException)
                }
            }
        }

        impl<T: ObjectType, R: StrongRef> Debug for $name<T, R> {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                f.write_str(&object_to_string(self.as_raw()))
            }
        }

        impl<T: ObjectType, R: StrongRef> Display for $name<T, R> {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                f.write_str(&object_to_string(self.as_raw()))
            }
        }

        impl<T: ObjectType, R: Ref> PartialEq for $name<T, R> {
            fn eq(&self, other: &Self) -> bool {
                ref_equal(self.as_raw(), other.as_raw())
            }
        }
    };
}

impl_common!(Class);
impl_common!(Object);

impl<'ctx, T: ObjectType> Class<T, Local<'ctx>> {
    pub fn find_class(ctx: &'ctx Context) -> Result<Self, LocalObject<'ctx, Throwable>> {
        fn class_name_of(signature: &Signature) -> Cow<'static, str> {
            match signature {
                Signature::Void => Cow::Borrowed("V"),
                Signature::Boolean => Cow::Borrowed("Z"),
                Signature::Byte => Cow::Borrowed("B"),
                Signature::Char => Cow::Borrowed("C"),
                Signature::Short => Cow::Borrowed("S"),
                Signature::Int => Cow::Borrowed("I"),
                Signature::Long => Cow::Borrowed("J"),
                Signature::Float => Cow::Borrowed("F"),
                Signature::Double => Cow::Borrowed("D"),
                Signature::Object(cls) => Cow::Borrowed(cls),
                Signature::Array(s) => Cow::Owned(format!("[{}", s)),
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

impl<T: ObjectType, R: StrongRef> Class<T, R> {
    pub fn is_assignable_from<ST: ObjectType, SR: StrongRef>(&self, ctx: &Context, superclass: &Class<ST, SR>) -> bool {
        unsafe { ctx.is_assignable_from(self.as_raw(), superclass.as_raw()) }
    }
}

pub trait Args<'a>: 'a {
    type Array<T>
    where
        T: 'a;

    fn signatures() -> Self::Array<Signature>
    where
        Self::Array<Signature>: AsRef<[Signature]>;

    fn into_raw(self) -> Self::Array<CallArg<'a>>
    where
        Self::Array<CallArg<'a>>: CallArgs;
}

fn call_method<'ctx, 't, 'a, const STATIC: bool, T, R, A>(
    ctx: &'ctx Context,
    this: &T,
    name: &'static str,
    args: A,
) -> Result<R, LocalObject<'ctx, Throwable>>
where
    T: AsRaw,
    T::Raw: StrongRef,
    R: Type,
    R: FromRaw,
    R::Raw: CallResult<'ctx>,
    A: Args<'a>,
    A::Array<Signature>: AsRef<[Signature]>,
    A::Array<CallArg<'a>>: CallArgs,
{
    let method: Method<STATIC> = if STATIC {
        resolver::find_method::<STATIC, _, A, R>(ctx, this.as_raw(), name)?
    } else {
        let class = ctx.get_object_class(this.as_raw());

        resolver::find_method::<STATIC, _, A, R>(ctx, &class, name)?
    };

    let raw_args = args.into_raw();

    unsafe { ctx.call_method(this.as_raw(), method, raw_args).map(|v| R::from_raw(v)) }
}

impl<T: ObjectType, R: StrongRef> Object<T, R> {
    pub fn call_method<'ctx, 'a, V, A>(
        &self,
        ctx: &'ctx Context,
        name: &'static str,
        args: A,
    ) -> Result<V, LocalObject<'ctx, Throwable>>
    where
        V: Type,
        V: FromRaw,
        V::Raw: CallResult<'ctx>,
        A: Args<'a>,
        A::Array<Signature>: AsRef<[Signature]>,
        A::Array<CallArg<'a>>: CallArgs,
    {
        call_method::<false, _, _, _>(ctx, self, name, args)
    }
}

impl<T: ObjectType, R: StrongRef> Class<T, R> {
    pub fn call_method<'ctx, 'a, V, A>(
        &self,
        ctx: &'ctx Context,
        name: &'static str,
        args: A,
    ) -> Result<V, LocalObject<'ctx, Throwable>>
    where
        V: Type,
        V: FromRaw,
        V::Raw: CallResult<'ctx>,
        A: Args<'a>,
        A::Array<Signature>: AsRef<[Signature]>,
        A::Array<CallArg<'a>>: CallArgs,
    {
        call_method::<true, _, _, _>(ctx, self, name, args)
    }
}

impl<T: ObjectType, R: StrongRef> Class<T, R> {
    pub fn new_object<'ctx, 'args, A>(
        &self,
        ctx: &'ctx Context,
        args: A,
    ) -> Result<LocalObject<'ctx, T>, LocalObject<'ctx, Throwable>>
    where
        A: Args<'args>,
        A::Array<Signature>: AsRef<[Signature]>,
        A::Array<CallArg<'args>>: CallArgs,
        LocalObject<'ctx, T>: Raw<Raw = Local<'ctx>> + FromRaw,
    {
        let method: Method<false> = resolver::find_method::<false, _, A, ()>(ctx, self.as_raw(), "<init>")?;

        let raw_args = args.into_raw();
        unsafe { ctx.new_object(self.as_raw(), method, raw_args).map(|v| Object::from_raw(v)) }
    }
}

fn get_field<'ctx, const STATIC: bool, T, R>(
    ctx: &'ctx Context,
    this: &T,
    name: &'static str,
) -> Result<R, LocalObject<'ctx, Throwable>>
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

impl<T: ObjectType, R: StrongRef> Object<T, R> {
    pub fn get_field<'ctx, V>(&self, ctx: &'ctx Context, name: &'static str) -> Result<V, LocalObject<'ctx, Throwable>>
    where
        V: FromRaw + Type,
        V::Raw: GetReturn<'ctx>,
    {
        get_field::<false, _, _>(ctx, self, name)
    }
}

impl<T: ObjectType, R: StrongRef> Class<T, R> {
    pub fn get_field<'ctx, V>(&self, ctx: &'ctx Context, name: &'static str) -> Result<V, LocalObject<'ctx, Throwable>>
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
) -> Result<(), LocalObject<'ctx, Throwable>>
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

impl<T: ObjectType, R: StrongRef> Object<T, R> {
    pub fn set_field<'ctx, V>(&self, ctx: &'ctx Context, name: &'static str, value: V) -> Result<(), LocalObject<'ctx, Throwable>>
    where
        V: IntoRaw + Type,
        V::Raw: SetArg,
    {
        set_field::<false, _, _>(ctx, self, name, value)
    }
}

impl<T: ObjectType, R: StrongRef> Class<T, R> {
    pub fn set_field<'ctx, V>(&self, ctx: &'ctx Context, name: &'static str, value: V) -> Result<(), LocalObject<'ctx, Throwable>>
    where
        V: IntoRaw + Type,
        V::Raw: SetArg,
    {
        set_field::<true, _, _>(ctx, self, name, value)
    }
}
