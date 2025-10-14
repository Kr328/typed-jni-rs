use alloc::string::String;
use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    AsRaw, Class, Context, FromRaw, Global, IntoRaw, Local, LocalObject, Object, ObjectType, PrimitiveArrayElement,
    PrimitiveType, Ref, Signature, StrongRef, Type,
};

macro_rules! define_object_builtin {
    ($name:ident, $class:literal) => {
        pub struct $name;

        impl Type for $name {
            const SIGNATURE: Signature = Signature::Object($class);
        }

        impl ObjectType for $name {}
    };
}

define_object_builtin!(JavaThrowable, "java/lang/Throwable");

define_object_builtin!(JavaObject, "java/lang/Object");

define_object_builtin!(JavaString, "java/lang/String");

impl<R: StrongRef> Object<JavaString, R> {
    pub fn get_string(&self, ctx: &Context) -> String {
        unsafe { ctx.get_string(self.as_raw()) }
    }
}

impl<'ctx> Object<JavaString, Local<'ctx>> {
    pub fn new_string(ctx: &'ctx Context, s: impl AsRef<str>) -> Self {
        unsafe { Self::from_raw(ctx.new_string(s)) }
    }
}

pub struct Array<T: Type>(PhantomData<T>);

impl<T: Type> Type for Array<T> {
    const SIGNATURE: Signature = Signature::Array(&T::SIGNATURE);
}

impl<T: Type> ObjectType for Array<T> {}

impl<'ctx, T: Type + ObjectType> Object<Array<T>, Local<'ctx>> {
    pub fn new<CR: StrongRef>(
        ctx: &'ctx Context,
        size: i32,
        class: &Class<T, CR>,
    ) -> Result<Self, LocalObject<'ctx, JavaThrowable>> {
        unsafe {
            ctx.new_object_array::<_, Global>(size, class.as_raw(), None)
                .map(|r| Object::from_raw(r))
                .map_err(|err| LocalObject::from_raw(err))
        }
    }

    pub fn new_with_initial<'a, CR: StrongRef, OR: StrongRef>(
        ctx: &'ctx Context,
        size: i32,
        class: &Class<T, CR>,
        initial: &Object<T, OR>,
    ) -> Result<Self, LocalObject<'ctx, JavaThrowable>> {
        unsafe {
            ctx.new_object_array(size, class.as_raw(), Some(initial.into_raw()))
                .map(|r| Object::from_raw(r))
                .map_err(|err| LocalObject::from_raw(err))
        }
    }
}

impl<'ctx, T: Type + PrimitiveType + PrimitiveArrayElement> Object<Array<T>, Local<'ctx>> {
    pub fn new_primitive(ctx: &'ctx Context, size: i32) -> Result<Self, LocalObject<'ctx, JavaThrowable>> {
        unsafe {
            ctx.new_primitive_array::<T>(size)
                .map(|r| Self::from_raw(r))
                .map_err(|err| LocalObject::from_raw(err))
        }
    }
}

impl<T: Type, R: StrongRef> Object<Array<T>, R> {
    pub fn length(&self, ctx: &Context) -> i32 {
        unsafe { ctx.get_array_length(self.as_raw()) }
    }
}

impl<T: Type + ObjectType, R: StrongRef> Object<Array<T>, R> {
    pub fn get_element<'ctx>(
        &self,
        ctx: &'ctx Context,
        index: i32,
    ) -> Result<Option<LocalObject<'ctx, T>>, LocalObject<'ctx, JavaThrowable>> {
        unsafe {
            Ok(Option::from_raw(
                ctx.get_object_array_element(self.as_raw(), index)
                    .map_err(|err| LocalObject::from_raw(err))?,
            ))
        }
    }

    pub fn set_element<'ctx, 'a, RV: Ref + 'a>(
        &self,
        ctx: &'ctx Context,
        index: i32,
        object: Option<&'a Object<T, RV>>,
    ) -> Result<(), LocalObject<'ctx, JavaThrowable>> {
        unsafe {
            ctx.set_object_array_element(self.as_raw(), index, object.into_raw())
                .map_err(|err| LocalObject::from_raw(err))
        }
    }
}

pub struct PrimitiveArrayElements<'a, T: PrimitiveType + PrimitiveArrayElement, R: StrongRef> {
    array: &'a Object<Array<T>, R>,
    buf: &'a mut [T],
}

impl<'a, T: PrimitiveType + PrimitiveArrayElement, R: StrongRef> Deref for PrimitiveArrayElements<'a, T, R> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.buf
    }
}

impl<'a, T: PrimitiveType + PrimitiveArrayElement, R: StrongRef> DerefMut for PrimitiveArrayElements<'a, T, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buf
    }
}

impl<'a, T: PrimitiveType + PrimitiveArrayElement, R: StrongRef> Drop for PrimitiveArrayElements<'a, T, R> {
    fn drop(&mut self) {
        Context::with_attached(|ctx| unsafe { ctx.release_primitive_array_elements(self.array.as_raw(), self.buf, false) })
    }
}

impl<'a, T: PrimitiveType + PrimitiveArrayElement, R: StrongRef> PrimitiveArrayElements<'a, T, R> {
    pub fn commit(self) {
        Context::with_attached(|ctx| unsafe { ctx.release_primitive_array_elements(self.array.as_raw(), self.buf, true) });

        core::mem::forget(self)
    }
}

impl<T: Type + PrimitiveType + PrimitiveArrayElement, R: StrongRef> Object<Array<T>, R> {
    pub fn get_elements<'b>(&'b self, ctx: &'b Context) -> PrimitiveArrayElements<'b, T, R> {
        unsafe {
            let buf = ctx.get_primitive_array_elements(self.as_raw());

            PrimitiveArrayElements { array: self, buf }
        }
    }

    pub fn get_region<'ctx>(
        &self,
        ctx: &'ctx Context,
        offset: i32,
        buf: &mut [T],
    ) -> Result<(), LocalObject<'ctx, JavaThrowable>> {
        unsafe {
            ctx.get_primitive_array_region(self.as_raw(), offset, buf)
                .map_err(|err| LocalObject::from_raw(err))
        }
    }

    pub fn set_region<'ctx>(&self, ctx: &'ctx Context, offset: i32, buf: &[T]) -> Result<(), LocalObject<'ctx, JavaThrowable>> {
        unsafe {
            ctx.set_primitive_array_region(self.as_raw(), offset, buf)
                .map_err(|err| LocalObject::from_raw(err))
        }
    }
}

pub struct UByteArrayElements<'b, R: StrongRef> {
    array: &'b Object<Array<i8>, R>,
    buf: &'b mut [u8],
}

impl<'b, R: StrongRef> Deref for UByteArrayElements<'b, R> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.buf
    }
}

impl<'b, R: StrongRef> DerefMut for UByteArrayElements<'b, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buf
    }
}

impl<'b, R: StrongRef> Drop for UByteArrayElements<'b, R> {
    fn drop(&mut self) {
        Context::with_attached(|ctx| unsafe {
            ctx.release_primitive_array_elements(
                self.array.as_raw(),
                core::slice::from_raw_parts_mut(self.buf.as_mut_ptr() as *mut i8, self.buf.len()),
                false,
            )
        })
    }
}

impl<'b, R: StrongRef> UByteArrayElements<'b, R> {
    pub fn commit(self) {
        Context::with_attached(|ctx| unsafe {
            ctx.release_primitive_array_elements(
                self.array.as_raw(),
                core::slice::from_raw_parts_mut(self.buf.as_mut_ptr() as *mut i8, self.buf.len()),
                true,
            )
        });

        core::mem::forget(self)
    }
}

impl<R: StrongRef> Object<Array<i8>, R> {
    pub fn get_bytes_elements<'b>(&'b self, ctx: &'b Context) -> UByteArrayElements<'b, R> {
        unsafe {
            let buf: &mut [i8] = ctx.get_primitive_array_elements(self.as_raw());

            UByteArrayElements {
                array: self,
                buf: core::slice::from_raw_parts_mut(buf.as_mut_ptr() as _, buf.len()),
            }
        }
    }

    pub fn get_bytes_region<'ctx>(
        &self,
        ctx: &'ctx Context,
        offset: i32,
        buf: &mut [u8],
    ) -> Result<(), LocalObject<'ctx, JavaThrowable>> {
        unsafe {
            ctx.get_primitive_array_region(
                self.as_raw(),
                offset,
                core::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut i8, buf.len()),
            )
            .map_err(|err| LocalObject::from_raw(err))
        }
    }

    pub fn set_bytes_region<'ctx>(
        &self,
        ctx: &'ctx Context,
        offset: i32,
        buf: &[u8],
    ) -> Result<(), LocalObject<'ctx, JavaThrowable>> {
        unsafe {
            ctx.set_primitive_array_region(
                self.as_raw(),
                offset,
                core::slice::from_raw_parts(buf.as_ptr() as *const i8, buf.len()),
            )
            .map_err(|err| LocalObject::from_raw(err))
        }
    }
}
