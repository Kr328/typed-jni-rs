macro_rules! call {
    ($env_ptr:expr, $func_name:ident) => {
        { (**$env_ptr).$func_name.expect(concat!("BROKEN: function JNIEnv::", stringify!($func_name), " undefined"))($env_ptr as *const _ as *mut _) }
    };
    ($env_ptr:expr, $func_name:ident, $($args:expr),*) => {
        { (**$env_ptr).$func_name.expect(concat!("BROKEN: function JNIEnv::", stringify!($func_name), " undefined"))($env_ptr as *const _ as *mut _, $($args),*) }
    };
}

pub(crate) use call;
