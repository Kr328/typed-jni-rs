use crate::{FieldID, JNIEnv, LocalRef, StrongRef, helper::call};

macro_rules! define_get_set_field {
    ($get_name:ident, $set_name:ident, $typ:ty, $get_func:ident, $set_func:ident, $get_static_func:ident, $set_static_func:ident, $get_doc:literal, $set_doc:literal) => {
        impl<'vm> JNIEnv<'vm> {
            #[doc = $get_doc]
            pub unsafe fn $get_name<const STATIC: bool, R: StrongRef>(
                &self,
                obj: &R,
                field: FieldID<STATIC>,
            ) -> Result<$typ, LocalRef<'_>> {
                #[cfg(debug_assertions)]
                obj.enforce_valid_runtime(self);

                self.run_catch(|| unsafe {
                    if STATIC {
                        call!(
                            self.as_raw_ptr(),
                            $get_static_func,
                            obj.as_raw_ptr(),
                            field.as_raw_ptr()
                        )
                    } else {
                        call!(self.as_raw_ptr(), $get_func, obj.as_raw_ptr(), field.as_raw_ptr())
                    }
                })
            }

            #[doc = $set_doc]
            pub unsafe fn $set_name<const STATIC: bool, R: StrongRef>(
                &self,
                obj: &R,
                field: FieldID<STATIC>,
                value: $typ,
            ) -> Result<(), LocalRef<'_>> {
                #[cfg(debug_assertions)]
                obj.enforce_valid_runtime(self);

                self.run_catch(|| unsafe {
                    if STATIC {
                        call!(
                            self.as_raw_ptr(),
                            $set_static_func,
                            obj.as_raw_ptr(),
                            field.as_raw_ptr(),
                            value
                        )
                    } else {
                        call!(
                            self.as_raw_ptr(),
                            $set_func,
                            obj.as_raw_ptr(),
                            field.as_raw_ptr(),
                            value
                        )
                    }
                })
            }
        }
    };
}

define_get_set_field!(
    get_boolean_field,
    set_boolean_field,
    bool,
    GetBooleanField,
    SetBooleanField,
    GetStaticBooleanField,
    SetStaticBooleanField,
    "Get a boolean field of an object.\n\n# Safety\n\n- `field` must be a valid field of `obj`.",
    "Set a boolean field of an object.\n\n# Safety\n\n- `field` must be a valid field of `obj`."
);

define_get_set_field!(
    get_byte_field,
    set_byte_field,
    i8,
    GetByteField,
    SetByteField,
    GetStaticByteField,
    SetStaticByteField,
    "Get a byte field of an object.\n\n# Safety\n\n- `field` must be a valid field of `obj`.",
    "Set a byte field of an object.\n\n# Safety\n\n- `field` must be a valid field of `obj`."
);

define_get_set_field!(
    get_char_field,
    set_char_field,
    u16,
    GetCharField,
    SetCharField,
    GetStaticCharField,
    SetStaticCharField,
    "Get a char field of an object.\n\n# Safety\n\n- `field` must be a valid field of `obj`.",
    "Set a char field of an object.\n\n# Safety\n\n- `field` must be a valid field of `obj`."
);

define_get_set_field!(
    get_short_field,
    set_short_field,
    i16,
    GetShortField,
    SetShortField,
    GetStaticShortField,
    SetStaticShortField,
    "Get a short field of an object.\n\n# Safety\n\n- `field` must be a valid field of `obj`.",
    "Set a short field of an object.\n\n# Safety\n\n- `field` must be a valid field of `obj`."
);

define_get_set_field!(
    get_int_field,
    set_int_field,
    i32,
    GetIntField,
    SetIntField,
    GetStaticIntField,
    SetStaticIntField,
    "Get an int field of an object.\n\n# Safety\n\n- `field` must be a valid field of `obj`.",
    "Set an int field of an object.\n\n# Safety\n\n- `field` must be a valid field of `obj`."
);

define_get_set_field!(
    get_long_field,
    set_long_field,
    i64,
    GetLongField,
    SetLongField,
    GetStaticLongField,
    SetStaticLongField,
    "Get a long field of an object.\n\n# Safety\n\n- `field` must be a valid field of `obj`.",
    "Set a long field of an object.\n\n# Safety\n\n- `field` must be a valid field of `obj`."
);

define_get_set_field!(
    get_float_field,
    set_float_field,
    f32,
    GetFloatField,
    SetFloatField,
    GetStaticFloatField,
    SetStaticFloatField,
    "Get a float field of an object.\n\n# Safety\n\n- `field` must be a valid field of `obj`.",
    "Set a float field of an object.\n\n# Safety\n\n- `field` must be a valid field of `obj`."
);

define_get_set_field!(
    get_double_field,
    set_double_field,
    f64,
    GetDoubleField,
    SetDoubleField,
    GetStaticDoubleField,
    SetStaticDoubleField,
    "Get a double field of an object.\n\n# Safety\n\n- `field` must be a valid field of `obj`.",
    "Set a double field of an object.\n\n# Safety\n\n- `field` must be a valid field of `obj`."
);

impl<'vm> JNIEnv<'vm> {
    /// Get an object field of an object.
    ///
    /// # Safety
    ///
    /// - `obj` must be a valid object.
    /// - `field` must be a valid field of `obj`.
    pub unsafe fn get_object_field<const STATIC: bool, R: StrongRef>(
        &self,
        obj: &R,
        field: FieldID<STATIC>,
    ) -> Result<Option<LocalRef<'_>>, LocalRef<'_>> {
        #[cfg(debug_assertions)]
        obj.enforce_valid_runtime(self);

        self.run_catch(|| unsafe {
            let obj = if STATIC {
                call!(self.as_raw_ptr(), GetStaticObjectField, obj.as_raw_ptr(), field.as_raw_ptr())
            } else {
                call!(self.as_raw_ptr(), GetObjectField, obj.as_raw_ptr(), field.as_raw_ptr())
            };
            if obj.is_null() {
                None
            } else {
                Some(LocalRef::from_raw(self, obj))
            }
        })
    }

    /// Set an object field of an object.
    ///
    /// # Safety
    ///
    /// - `obj` must be a valid object.
    /// - `field` must be a valid field of `obj`.
    pub unsafe fn set_object_field<const STATIC: bool, R: StrongRef, VR: StrongRef>(
        &self,
        obj: &R,
        field: FieldID<STATIC>,
        value: Option<&VR>,
    ) -> Result<(), LocalRef<'_>> {
        #[cfg(debug_assertions)]
        obj.enforce_valid_runtime(self);

        #[cfg(debug_assertions)]
        if let Some(value) = value {
            value.enforce_valid_runtime(self);
        }

        self.run_catch(|| unsafe {
            if STATIC {
                call!(
                    self.as_raw_ptr(),
                    SetStaticObjectField,
                    obj.as_raw_ptr(),
                    field.as_raw_ptr(),
                    value.map_or(core::ptr::null_mut(), |v| v.as_raw_ptr())
                )
            } else {
                call!(
                    self.as_raw_ptr(),
                    SetObjectField,
                    obj.as_raw_ptr(),
                    field.as_raw_ptr(),
                    value.map_or(core::ptr::null_mut(), |v| v.as_raw_ptr())
                )
            }
        })
    }
}
