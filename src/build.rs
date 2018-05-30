extern crate cbindgen;

use std::env;

fn generate_ffi_headers() {

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::Builder::new()
      .with_crate(crate_dir)
      .with_namespaces(&["themed_icon_lookup"])
      .generate()
      .expect("Unable to generate bindings")
      .write_to_file("themed_icon_lookup.h");
}

fn main() {
    if cfg!(not(test)) {
      generate_ffi_headers();
    }
}