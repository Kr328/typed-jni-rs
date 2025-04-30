pub trait Raw {
    type Raw;
}

impl<V: Raw> Raw for Option<V> {
    type Raw = Option<V::Raw>;
}

pub trait AsRaw: Raw {
    fn as_raw(&self) -> &Self::Raw;
}

pub trait IntoRaw: Raw {
    fn into_raw(self) -> Self::Raw;
}

impl<V: IntoRaw> IntoRaw for Option<V> {
    fn into_raw(self) -> Self::Raw {
        self.map(|v| v.into_raw())
    }
}

pub trait FromRaw: Raw {
    unsafe fn from_raw(raw: Self::Raw) -> Self;
}

impl<V: FromRaw> FromRaw for Option<V> {
    unsafe fn from_raw(raw: Self::Raw) -> Self {
        unsafe { raw.map(|r| V::from_raw(r)) }
    }
}
