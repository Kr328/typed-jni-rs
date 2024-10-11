use crate::{Args, CallArg, IntoRaw, Signature, Type};

#[derive(Copy, Clone)]
pub struct NoArgs;

impl<'a> Args<'a> for NoArgs {
    type Array<T: 'a> = [T; 0];

    fn signatures() -> [Signature; 0] {
        []
    }

    fn into_raw(self) -> [CallArg<'a>; 0] {
        []
    }
}

macro_rules! impl_args {
    ($n:literal, $($args:ident),*) => {
        #[allow(unused_parens)]
        impl<'a, $($args: Type + IntoRaw + 'a),*> Args<'a> for ($($args),*) where $($args::Raw: Into<CallArg<'a>>),* {
            type Array<T: 'a> = [T;$n];

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

include!(concat!(env!("OUT_DIR"), "/args_impl.rs"));
