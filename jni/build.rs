use std::{fmt::Write, path::Path};

fn generate_args_impl(max_args_count: usize) {
    let mut file_content = String::new();

    for count in 1..=max_args_count {
        let mut args = String::new();

        for index in 1..=count {
            args.write_fmt(format_args!("A{},", index)).unwrap();
        }

        file_content
            .write_fmt(format_args!("impl_args!({}, {});\n", count, args.trim_end_matches(",")))
            .unwrap();
    }

    std::fs::write(
        Path::new(std::env::var("OUT_DIR").unwrap().as_str()).join("args_impl.rs"),
        file_content,
    )
    .unwrap();
}

fn main() {
    generate_args_impl(
        std::env::var("RUST_TYPED_JNI_MAX_ARGS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100usize),
    );
    println!("cargo:rerun-if-env-changed=RUST_TYPED_JNI_MAX_ARGS")
}
