use std::{env, path::PathBuf};

const VVENC_VERSION: &str = "1.8.0";

mod build {
    use super::*;
    use std::env;
    use std::path::Path;
    use std::process::{Command, Stdio};

    const REPO: &str = "https://github.com/fraunhoferhhi/vvenc.git";

    macro_rules! runner {
        ($cmd:expr, $($arg:expr),*) => {
            Command::new($cmd)
                $(.arg($arg))*
                .stderr(Stdio::inherit())
                .stdout(Stdio::inherit())
                .output()
                .expect(concat!($cmd, " failed"));

        };
    }

    pub fn build_from_src() -> PathBuf {
        let mut tag = "v".to_string();
        tag.push_str(VVENC_VERSION);

        let source = PathBuf::from(env::var("OUT_DIR").unwrap()).join("vvenc");

        if !Path::new(&source.join(".git")).exists() {
            runner!("git", "clone", "--depth", "1", "-b", tag, REPO, &source);
        } else {
            runner!(
                "git",
                "-C",
                source.to_str().unwrap(),
                "fetch",
                "--depth",
                "1",
                "origin",
                tag
            );
            runner!(
                "git",
                "-C",
                source.to_str().unwrap(),
                "checkout",
                "FETCH_HEAD"
            );
        }

        let install_dir = cmake::build(source);
        let pkg_config_dir = install_dir.join("lib/pkgconfig");
        pkg_config_dir
    }
}

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");

    // Try to find vvenc in the system. Else, build vvenc from source.
    let library = match pkg_config::Config::new()
        .atleast_version(VVENC_VERSION)
        .probe("libvvenc")
    {
        Ok(library) => library,
        Err(_) => {
            if env::var("VVENC_SYS_BUILD_DEP_FROM_SRC").is_err() {
                panic!(
                    "libvvenc not found in the system. To allow building it from source, \
                    set environment variable VVENC_SYS_BUILD_DEP_FROM_SRC=1"
                );
            }
            let pkg_config_dir = build::build_from_src();
            env::set_var("PKG_CONFIG_PATH", pkg_config_dir.as_os_str());
            pkg_config::Config::new()
                .atleast_version(VVENC_VERSION)
                .probe("libvvenc")
                .unwrap()
        }
    };

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(
            library
                .include_paths
                .iter()
                .map(|path| format!("-I{}", path.to_string_lossy())),
        )
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
