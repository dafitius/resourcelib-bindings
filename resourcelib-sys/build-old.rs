// mod build_old;

use std::env;
use std::path::{Path, PathBuf};
use bindgen::callbacks::{AttributeInfo, ParseCallbacks};
use cmake::Config;

#[derive(Debug)]
struct ResourceLibCallbacks;

impl ParseCallbacks for ResourceLibCallbacks {
    fn add_attributes(&self, _info: &AttributeInfo<'_>) -> Vec<String> {
        vec!["#[allow(non_snake_case)]".into()]

    }
}

fn main() {
    let zhmtools_path = Path::new("extern").join("ZHMTools").canonicalize().unwrap();
    let rlib_path = zhmtools_path.join("Libraries").join("ResourceLib");
    let rlib_include_path = rlib_path.join("Include");

    let opt_level = env::var("OPT_LEVEL").unwrap_or_else(|_| "0".to_string());

    let build_profile = if opt_level == "0" {
        "Debug"
    } else {
        "Release"
    };

    let targets = ["ResourceLib_HM2016", "ResourceLib_HM2", "ResourceLib_HM3"];

    let out_path = Config::new(&zhmtools_path)
        .profile(build_profile)
        .define("ZHM_BUILD_TOOLS", "OFF")
        .no_default_c_flags(true)
        .no_default_cxx_flags(true)
        .build_target(targets.join(";").as_str())
        .build();

    // Determine the output directory based on the platform and build profile
    let build_path = out_path.join("build");
    let lib_dir = if cfg!(windows) {
        build_path.join(build_profile)
    } else {
        build_path.clone()
    };

    println!(
        "cargo:rustc-link-search=native={}",
        lib_dir.display()
    );

    for target in &targets {
        println!("cargo:rustc-link-lib=dylib={}", target);
    }

    println!("cargo:include={}", rlib_include_path.display());

    let bindings = bindgen::Builder::default()
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++23")
        .clang_arg(format!("-I{}", rlib_include_path.display()))
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
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}