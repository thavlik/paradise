fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_header = "include/paradise.h";
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_include_guard("PARADISE_H")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_header);
    println!("cargo:rerun-if-changed=\"{}\"", out_header);
}
