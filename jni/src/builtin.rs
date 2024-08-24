use alloc::string::String;
use core::marker::PhantomData;

use crate::{
    typed::{Signature, Type},
    AsRaw, Class, Context, FromRaw, Global, IntoRaw, Object, ObjectType, PrimitiveArrayElement, PrimitiveType, Ref, StrongRef,
};

pub struct Throwable;

impl Type for Throwable {
    const SIGNATURE: Signature = Signature::Object("java/lang/Throwable");
}

impl ObjectType for Throwable {}

#[cfg(feature = "std")]
impl<'r, R: super::StrongRef> std::error::Error for Object<'r, Throwable, R> {}

pub struct JString;

impl Type for JString {
    const SIGNATURE: Signature = Signature::Object("java/lang/String");
}

impl ObjectType for JString {}

impl<'r, R: StrongRef> Object<'r, JString, R> {
    pub fn get_string(&self, ctx: &Context) -> String {
        unsafe { ctx.get_string(self.as_raw()) }
    }
}

impl<'ctx> Object<'ctx, JString> {
    pub fn new_string(ctx: &'ctx Context, s: impl AsRef<str>) -> Self {
        unsafe { Self::from_raw(ctx.new_string(s)) }
    }
}

pub struct Array<T: Type>(PhantomData<T>);

impl<T: Type> Type for Array<T> {
    const SIGNATURE: Signature = Signature::Array(&T::SIGNATURE);
}

impl<T: Type> ObjectType for Array<T> {}

impl<'ctx, T: Type> Object<'ctx, Array<T>> {
    pub fn primitive(ctx: &'ctx Context, size: i32) -> Result<Self, Object<'ctx, Throwable>>
    where
        T: PrimitiveType + PrimitiveArrayElement,
    {
        unsafe { ctx.new_primitive_array::<T>(size).map(|r| Self::from_raw(r)) }
    }

    pub fn new<CR: StrongRef>(ctx: &'ctx Context, size: i32, class: &Class<T, CR>) -> Result<Self, Object<'ctx, Throwable>>
    where
        T: ObjectType,
    {
        unsafe {
            ctx.new_object_array::<_, Global>(size, class.as_raw(), None)
                .map(|r| Object::from_raw(r))
        }
    }

    pub fn with_initial<'a, CR: StrongRef, OR: StrongRef>(
        ctx: &'ctx Context,
        size: i32,
        class: &Class<T, CR>,
        initial: &Object<T, OR>,
    ) -> Result<Self, Object<'ctx, Throwable>>
    where
        T: ObjectType,
    {
        unsafe {
            ctx.new_object_array(size, class.as_raw(), Some(initial.into_raw()))
                .map(|r| Object::from_raw(r))
        }
    }
}

impl<'r, T: Type, R: StrongRef> Object<'r, Array<T>, R> {
    pub fn length(&self, ctx: &Context) -> i32 {
        unsafe { ctx.get_array_length(self.as_raw()) }
    }

    pub fn get_region<'ctx>(&self, ctx: &'ctx Context, offset: i32, buf: &mut [T]) -> Result<(), Object<'ctx, Throwable>>
    where
        T: PrimitiveType + PrimitiveArrayElement,
    {
        unsafe { ctx.get_primitive_array_region(self.as_raw(), offset, buf) }
    }

    pub fn set_region<'ctx>(&self, ctx: &'ctx Context, offset: i32, buf: &[T]) -> Result<(), Object<'ctx, Throwable>>
    where
        T: PrimitiveType + PrimitiveArrayElement,
    {
        unsafe { ctx.set_primitive_array_region(self.as_raw(), offset, buf) }
    }

    pub fn get_element<'ctx>(&self, ctx: &'ctx Context, index: i32) -> Result<Option<Object<'ctx, T>>, Object<'ctx, Throwable>>
    where
        T: ObjectType,
    {
        unsafe { Ok(Option::from_raw(ctx.get_object_array_element(self.as_raw(), index)?)) }
    }

    pub fn set_element<'ctx, 'a, RV: Ref + 'a>(
        &self,
        ctx: &'ctx Context,
        index: i32,
        object: Option<&'a Object<T, RV>>,
    ) -> Result<(), Object<'ctx, Throwable>>
    where
        T: ObjectType,
    {
        unsafe { ctx.set_object_array_element(self.as_raw(), index, object.into_raw()) }
    }
}
