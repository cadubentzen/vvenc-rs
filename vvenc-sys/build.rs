use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");

    pkg_config::Config::new().atleast_version("1.7.0").probe("libvvenc").unwrap();

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .allowlist_type("vvenc.*")
        .allowlist_type("ErrorCodes")
        .allowlist_function("vvenc_.*")
        .allowlist_var("VVENC.*")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}