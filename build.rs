extern crate bindgen;
use std::env;
use std::path::PathBuf;

fn main() {
  // Tell cargo to tell rustc to link the system bzip2
  // shared library.
  // println!("cargo:rustc-link-lib=nixutilc");
  // println!("cargo:rustc-link-lib=nixstorec");
  // println!("cargo:rustc-link-lib=nixexprc");
  let nix_expr_c = pkg_config::probe_library("nix-expr-c").unwrap();
  let nix_store_c = pkg_config::probe_library("nix-store-c").unwrap();
  let bindings = bindgen::Builder::default()
    .clang_args(nix_expr_c.include_paths.iter().map(|path| format!("-I{}", path.to_string_lossy())))
    .clang_args(nix_store_c.include_paths.iter().map(|path| format!("-I{}", path.to_string_lossy())))
    .header("src/wrapper.h")
    // Finish the builder and generate the bindings.
    .generate()
    // Unwrap the Result and panic on failure.
    .expect("Unable to generate bindings");

  // Write the bindings to the $OUT_DIR/bindings.rs file.
  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("Couldn't write bindings!");
}
