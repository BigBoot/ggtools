use built::write_built_file_with_opts;
use std::{env, path::Path};

fn main() {
    let mut opts = built::Options::default();
    opts.set_git(true);
    opts.set_time(true);

    let src = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dst = Path::new(&env::var("OUT_DIR").unwrap()).join("built.rs");
    write_built_file_with_opts(&opts, &Path::new(&src), &dst).expect("Failed to acquire build-time information");
}
