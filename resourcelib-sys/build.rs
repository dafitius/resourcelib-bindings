// mod build;

use std::{env, fs, io};
use std::error::Error;
use std::path::{Path, PathBuf};
use bindgen::{BindgenError, Bindings};
use bindgen::callbacks::{AttributeInfo, ParseCallbacks};
use cmake::Config;

#[derive(Debug)]
struct ResourceLibCallbacks;

impl ParseCallbacks for ResourceLibCallbacks {
    fn add_attributes(&self, _info: &AttributeInfo<'_>) -> Vec<String> {
        vec!["#[allow(non_snake_case)]".into()]

    }
}

macro_rules! warn {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

macro_rules! err {
    ($($tokens: tt)*) => {
        panic!("{}", format!($($tokens)*))
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs"); //prevent this file from running every time

    let targets = ["ResourceLib_HM2016", "ResourceLib_HM2", "ResourceLib_HM3"];

    let mut zhmtools_path = Path::new("../extern").join("ZHMTools").canonicalize().unwrap();
    if cfg!(windows) { //We need to prevent a \\?\ prefix on the path, because MSVC compilers don't support UNC paths
        zhmtools_path = Path::new(&zhmtools_path.display().to_string().replace("\\\\?\\", "")).to_path_buf();
    }
    let local_rlib_path = zhmtools_path.join("Libraries").join("ResourceLib");

    let lib_path_override = env::var("RESOURCELIB_RELEASE_OVERRIDE").ok();

    let custom_install = lib_path_override.map(|path| PathBuf::from(&path));

    let lib_path = match &custom_install{
        Some(path) => {
            path.to_owned()
        },
        None => {
            build_from_source(zhmtools_path, &targets).unwrap_or_else(|e|err!("{}", e))
        }
    };

    let rlib_include_path =  match &custom_install {
        Some(path) => {
            path.to_owned()
        },
        None => {local_rlib_path}
    }.join("Include");

    println!("cargo:lib_path={}", lib_path.display());

    println!(
        "cargo:rustc-link-search=native={}",
        lib_path.display()
    );

    for target in &targets {
        println!("cargo:rustc-link-lib=dylib={}", target);
    }

    let out_path = env::var("OUT_DIR").map(|dir| PathBuf::from(dir)).expect("OUT_DIR not set");
    let bindings = generate_bindings(rlib_include_path).expect("Unable to generate bindings");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn generate_bindings<P: AsRef<Path>>(include_path: P) -> Result<Bindings, BindgenError> {
   bindgen::Builder::default()
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++23")
        .clang_arg(format!("-I{}", include_path.as_ref().display()))
        .header("wrapper.h")
        .parse_callbacks(Box::new(ResourceLibCallbacks))
        // Allowlist functions
        .allowlist_function("HM\\d+_GetConverterForResource")
        .allowlist_function("HM\\d+_GetGeneratorForResource")
        .allowlist_function("HM\\d+_GetSupportedResourceTypes")
        .allowlist_function("HM\\d+_FreeSupportedResourceTypes")
        .allowlist_function("HM\\d+_IsResourceTypeSupported")
        .allowlist_function("HM\\d+_GameStructToJson")
        .allowlist_function("HM\\d+_JsonToGameStruct")
        .allowlist_function("HM\\d+_FreeJsonString")
        .allowlist_function("HM\\d+_GetPropertyName")

        // Allowlist types
        .allowlist_type("ResourceConverter")
        .allowlist_type("ResourceGenerator")
        .allowlist_type("JsonString")
        .allowlist_type("ResourceMem")
        .allowlist_type("ResourceTypesArray")
        .allowlist_type("StringView")

        // Blocklist items
        .blocklist_item("__.*")
        .blocklist_item("_.*")
        .generate()
}

fn build_from_source<P: AsRef<Path>>(path: P, targets: &[&str; 3]) -> Result<PathBuf, Box<dyn Error>> {

    let opt_level = env::var("OPT_LEVEL").unwrap_or_else(|_| "0".to_string());

    let build_profile = if opt_level == "0" {
        "Debug"
    } else {
        "Release"
    };


    let mut config = Config::new(&path);
    let mut config = config
        .profile(build_profile)
        .define("ZHM_BUILD_TOOLS", "OFF");

    // Use platform-specific compiler flags
    if cfg!(windows) {
        config = config
            .cxxflag("/std:c++latest")
            .cxxflag("/EHsc");
    } else {
        config = config
            .no_default_flags(true);
    }

    let out_path = config.build_target(targets.join(";").as_str()).build();


    // Determine the output directory based on the platform and build profile
    let build_path = out_path.join("build");
    let lib_dir = if cfg!(windows) {
        build_path.join(build_profile)
    } else {
        build_path.clone()
    };

    Ok(lib_dir)
}