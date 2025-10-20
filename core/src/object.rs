use crate::{JNIEnv, LocalRef, Ref, StrongRef, helper::call, sys};

impl<'vm> JNIEnv<'vm> {
    /// Get the class of an object.
    pub fn get_object_class<R: StrongRef>(&self, object: &R) -> LocalRef<'_> {
        #[cfg(debug_assertions)]
        object.enforce_valid_runtime(self);

        let obj = self
            .run_catch(|| unsafe { call!(self.as_raw_ptr(), GetObjectClass, object.as_raw_ptr()) })
            .expect("BROKEN: cannot get class of object");

        unsafe { LocalRef::from_raw(self, obj) }
    }

    /// Get the super class of a class.
    ///
    /// # Safety
    /// - `class` must be a valid class.
    pub unsafe fn get_super_class<R: StrongRef>(&self, class: &R) -> LocalRef<'_> {
        #[cfg(debug_assertions)]
        class.enforce_valid_runtime(self);

        let super_class = self
            .run_catch(|| unsafe { call!(self.as_raw_ptr(), GetSuperclass, class.as_raw_ptr()) })
            .expect("BROKEN: cannot get super class of class");

        unsafe { LocalRef::from_raw(self, super_class) }
    }

    /// Check if an object is an instance of a class.
    ///
    /// # Safety
    /// - `object` must be a valid object.
    /// - `class` must be a valid class.
    pub unsafe fn is_instance_of<R1: StrongRef, R2: StrongRef>(&self, object: &R1, class: &R2) -> bool {
        #[cfg(debug_assertions)]
        object.enforce_valid_runtime(self);

        #[cfg(debug_assertions)]
        class.enforce_valid_runtime(self);

        self.run_catch(|| unsafe {
            let ret = call!(self.as_raw_ptr(), IsInstanceOf, object.as_raw_ptr(), class.as_raw_ptr());
            ret != sys::JNI_FALSE
        })
        .expect("BROKEN: cannot check instance of object")
    }

    /// Check if a class is assignable from another class.
    ///
    /// # Safety
    /// - `class` must be a valid class.
    /// - `super_class` must be a valid class.
    pub unsafe fn is_assignable_from<R1: StrongRef, R2: StrongRef>(&self, class: &R1, super_class: &R2) -> bool {
        #[cfg(debug_assertions)]
        class.enforce_valid_runtime(self);

        #[cfg(debug_assertions)]
        super_class.enforce_valid_runtime(self);

        self.run_catch(|| unsafe {
            let ret = call!(
                self.as_raw_ptr(),
                IsAssignableFrom,
                class.as_raw_ptr(),
                super_class.as_raw_ptr()
            );
            ret != sys::JNI_FALSE
        })
        .expect("BROKEN: cannot check assignability of classes")
    }

    /// Check if two objects are the same.
    pub fn is_same_object<R1: Ref, R2: Ref>(&self, object1: Option<&R1>, object2: Option<&R2>) -> bool {
        #[cfg(debug_assertions)]
        if let Some(object1) = object1 {
            object1.enforce_valid_runtime(self);
        }

        #[cfg(debug_assertions)]
        if let Some(object2) = object2 {
            object2.enforce_valid_runtime(self);
        }

        self.run_catch(|| unsafe {
            let ret = call!(
                self.as_raw_ptr(),
                IsSameObject,
                object1.map(|o| o.as_raw_ptr()).unwrap_or(core::ptr::null_mut()),
                object2.map(|o| o.as_raw_ptr()).unwrap_or(core::ptr::null_mut())
            );
            ret != sys::JNI_FALSE
        })
        .expect("BROKEN: cannot check sameness of objects")
    }
}
