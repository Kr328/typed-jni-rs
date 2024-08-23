use alloc::string::String;
use core::marker::PhantomData;

use crate::{
    context::{Context, PrimitiveArrayElement},
    reference::{Local, Ref, StrongRef},
    throwable::JavaThrowable,
    typed::{Class, FromRaw, IntoRaw, RefConvert, Signature, This, Type, Value},
    Global,
};

#[repr(transparent)]
pub struct JavaString<const STATIC: bool, R: Ref>(R);

impl<const STATIC: bool, R: Ref> RefConvert for JavaString<STATIC, R> {
    type Ref = R;
    type Converted<RR: Ref> = JavaString<STATIC, RR>;

    fn as_raw_ref(&self) -> &Self::Ref {
        &self.0
    }

    unsafe fn from_raw_ref<RR: Ref>(r: RR) -> Self::Converted<RR> {
        JavaString(r)
    }
}

impl<const STATIC: bool, R: Ref> Value for JavaString<STATIC, R> {
    type Raw = R;
}

impl<const STATIC: bool, R: Ref> FromRaw for JavaString<STATIC, R> {
    unsafe fn from_raw(raw: Self::Raw) -> Self {
        Self(raw)
    }
}

impl<'a, const STATIC: bool, R: Ref> Value for &'a JavaString<STATIC, R> {
    type Raw = &'a R;
}

impl<'a, const STATIC: bool, R: Ref> IntoRaw for &'a JavaString<STATIC, R> {
    fn into_raw(self) -> Self::Raw {
        &self.0
    }
}

impl<R: Ref> Type for JavaString<false, R> {
    const SIGNATURE: Signature = Signature::Object("java/lang/String");
}

impl<R: Ref> Type for JavaString<true, R> {
    const SIGNATURE: Signature = Signature::Object("java/lang/Class");
}

impl<const STATIC: bool, R: StrongRef> This<STATIC> for JavaString<STATIC, R> {
    type Ref = R;

    fn as_ref(&self) -> &Self::Ref {
        &self.0
    }
}

impl<R: StrongRef> Class for JavaString<true, R> {
    type Object<RR: Ref> = JavaString<false, RR>;
}

impl<R: StrongRef> JavaString<false, R> {
    pub fn to_string(&self, ctx: &Context) -> String {
        unsafe { ctx.get_string(self.as_ref()) }
    }
}

impl<'ctx> JavaString<false, Local<'ctx>> {
    pub fn from_string(ctx: &'ctx Context, s: impl AsRef<str>) -> Self {
        unsafe { Self::from_raw(ctx.new_string(s)) }
    }
}

#[repr(transparent)]
pub struct JavaPrimitiveArray<const STATIC: bool, T: PrimitiveArrayElement, R: Ref>(R, PhantomData<T>);

impl<const STATIC: bool, T: PrimitiveArrayElement, R: Ref> RefConvert for JavaPrimitiveArray<STATIC, T, R> {
    type Ref = R;
    type Converted<RR: Ref> = JavaPrimitiveArray<STATIC, T, RR>;

    fn as_raw_ref(&self) -> &Self::Ref {
        &self.0
    }

    unsafe fn from_raw_ref<RR: Ref>(r: RR) -> Self::Converted<RR> {
        JavaPrimitiveArray(r, PhantomData)
    }
}

impl<const STATIC: bool, T: PrimitiveArrayElement, R: Ref> Value for JavaPrimitiveArray<STATIC, T, R> {
    type Raw = R;
}

impl<const STATIC: bool, T: PrimitiveArrayElement, R: Ref> FromRaw for JavaPrimitiveArray<STATIC, T, R> {
    unsafe fn from_raw(raw: Self::Raw) -> Self {
        Self(raw, PhantomData)
    }
}

impl<'a, const STATIC: bool, T: PrimitiveArrayElement, R: Ref> Value for &'a JavaPrimitiveArray<STATIC, T, R> {
    type Raw = &'a R;
}

impl<'a, const STATIC: bool, T: PrimitiveArrayElement, R: Ref> IntoRaw for &'a JavaPrimitiveArray<STATIC, T, R> {
    fn into_raw(self) -> Self::Raw {
        &self.0
    }
}

impl<R: Ref, T: Type + PrimitiveArrayElement> Type for JavaPrimitiveArray<false, T, R> {
    const SIGNATURE: Signature = Signature::Array(&T::SIGNATURE);
}

impl<R: Ref, T: PrimitiveArrayElement> Type for JavaPrimitiveArray<true, T, R> {
    const SIGNATURE: Signature = Signature::Object("java/lang/Class");
}

impl<const STATIC: bool, T: PrimitiveArrayElement, R: StrongRef> This<STATIC> for JavaPrimitiveArray<STATIC, T, R> {
    type Ref = R;

    fn as_ref(&self) -> &Self::Ref {
        &self.0
    }
}

impl<R: StrongRef, T: PrimitiveArrayElement> Class for JavaPrimitiveArray<true, T, R> {
    type Object<RR: Ref> = JavaPrimitiveArray<false, T, RR>;
}

impl<T: PrimitiveArrayElement, R: StrongRef> JavaPrimitiveArray<false, T, R> {
    pub fn length(&self, ctx: &Context) -> i32 {
        unsafe { ctx.get_array_length(&self.0) }
    }
}

impl<'ctx, T: PrimitiveArrayElement> JavaPrimitiveArray<false, T, Local<'ctx>> {
    pub fn new(ctx: &'ctx Context, size: i32) -> Result<Self, JavaThrowable<false, Local<'ctx>>> {
        unsafe { ctx.new_primitive_array::<T>(size).map(|r| Self(r, PhantomData)) }
    }
}

impl<T: PrimitiveArrayElement, R: StrongRef> JavaPrimitiveArray<false, T, R> {
    pub fn get_region<'ctx>(
        &self,
        ctx: &'ctx Context,
        offset: i32,
        buf: &mut [T],
    ) -> Result<(), JavaThrowable<false, Local<'ctx>>> {
        unsafe { ctx.get_primitive_array_region(&self.0, offset, buf) }
    }

    pub fn set_region<'ctx>(&self, ctx: &'ctx Context, offset: i32, buf: &[T]) -> Result<(), JavaThrowable<false, Local<'ctx>>> {
        unsafe { ctx.set_primitive_array_region(&self.0, offset, buf) }
    }
}

#[repr(transparent)]
pub struct JavaArray<const STATIC: bool, C: Class, R: Ref>(R, PhantomData<C>);

impl<const STATIC: bool, C: Class, R: Ref> RefConvert for JavaArray<STATIC, C, R> {
    type Ref = R;
    type Converted<RR: Ref> = JavaArray<STATIC, C, RR>;

    fn as_raw_ref(&self) -> &Self::Ref {
        &self.0
    }

    unsafe fn from_raw_ref<RR: Ref>(r: RR) -> Self::Converted<RR> {
        JavaArray(r, PhantomData)
    }
}

impl<const STATIC: bool, C: Class, R: Ref> Value for JavaArray<STATIC, C, R> {
    type Raw = R;
}

impl<const STATIC: bool, C: Class, R: Ref> FromRaw for JavaArray<STATIC, C, R> {
    unsafe fn from_raw(raw: Self::Raw) -> Self {
        Self(raw, PhantomData)
    }
}

impl<'a, const STATIC: bool, C: Class, R: Ref> Value for &'a JavaArray<STATIC, C, R> {
    type Raw = &'a R;
}

impl<'a, const STATIC: bool, C: Class, R: Ref> IntoRaw for &'a JavaArray<STATIC, C, R> {
    fn into_raw(self) -> Self::Raw {
        &self.0
    }
}

impl<R: Ref, C: Class> Type for JavaArray<false, C, R>
where
    C::Object<Global>: Type,
{
    const SIGNATURE: Signature = Signature::Array(&C::Object::<Global>::SIGNATURE);
}

impl<R: Ref, C: Class> Type for JavaArray<true, C, R> {
    const SIGNATURE: Signature = Signature::Object("java/lang/Class");
}

impl<const STATIC: bool, C: Class, R: StrongRef> This<STATIC> for JavaArray<STATIC, C, R> {
    type Ref = R;

    fn as_ref(&self) -> &Self::Ref {
        &self.0
    }
}

impl<R: StrongRef, C: Class> Class for JavaArray<true, C, R> {
    type Object<RR: Ref> = JavaArray<false, C, RR>;
}

impl<C: Class, R: StrongRef> JavaArray<false, C, R> {
    pub fn length(&self, ctx: &Context) -> i32 {
        unsafe { ctx.get_array_length(&self.0) }
    }
}

impl<'ctx, C: Class + This<true>> JavaArray<false, C, Local<'ctx>> {
    pub fn new<'a, RR: Ref + 'a>(
        ctx: &'ctx Context,
        size: i32,
        class: &C,
        initial: Option<&'a C::Object<RR>>,
    ) -> Result<Self, JavaThrowable<false, Local<'ctx>>>
    where
        &'a C::Object<RR>: Value<Raw = &'a RR> + IntoRaw,
    {
        unsafe {
            ctx.new_object_array(size, class.as_ref(), initial.into_raw())
                .map(|r| Self(r, PhantomData))
        }
    }
}

impl<C: Class + This<true>, R: StrongRef> JavaArray<false, C, R> {
    pub fn get_element<'ctx>(
        &self,
        ctx: &'ctx Context,
        index: i32,
    ) -> Result<Option<C::Object<Local<'ctx>>>, JavaThrowable<false, Local<'ctx>>>
    where
        C::Object<Local<'ctx>>: Value<Raw = Local<'ctx>> + FromRaw,
    {
        unsafe { Ok(Option::from_raw(ctx.get_object_array_element(&self.0, index)?)) }
    }

    pub fn set_element<'a, RR: Ref + 'a>(
        &self,
        ctx: &'a Context,
        index: i32,
        object: Option<&'a C::Object<RR>>,
    ) -> Result<(), JavaThrowable<false, Local<'a>>>
    where
        &'a C::Object<RR>: Value<Raw = &'a RR> + IntoRaw,
    {
        unsafe { ctx.set_object_array_element(&self.0, index, object.into_raw()) }
    }
}
