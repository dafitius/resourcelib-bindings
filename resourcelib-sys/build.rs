// mod build;

use std::{env, fs, io};
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

macro_rules! warn {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let rlib_path = Path::new("extern").join("ResourceLib").canonicalize().unwrap();
    let rlib_lib_path = if cfg!(windows) {
        rlib_path.join("ResourceLib-win-x64")
    }else{
        rlib_path.join("ResourceLib-linux-x64")
    };

    copy_shared_libraries(&rlib_lib_path, &out_path).unwrap();

    println!("cargo:lib_path={}", out_path.display());
    
    println!(
        "cargo:rustc-link-search=native={}",
        out_path.display()
    );

    let targets = ["ResourceLib_HM2016", "ResourceLib_HM2", "ResourceLib_HM3"];
    for target in &targets {
        println!("cargo:rustc-link-lib=dylib={}", target);
    }

    let rlib_include_path = rlib_lib_path.join("Include");

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

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

}

fn copy_shared_libraries<P: AsRef<Path>, Q: AsRef<Path>>(
    source: P,
    destination: Q,
) -> io::Result<()> {
    let source_path = source.as_ref();
    let destination_path = destination.as_ref();

    if !source_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source directory {:?} does not exist.", source_path),
        ));
    }

    if !source_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{:?} is not a directory.", source_path),
        ));
    }

    if !destination_path.exists() {
        fs::create_dir_all(destination_path)?;
    } else if !destination_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{:?} is not a directory.", destination_path),
        ));
    }

    for entry in fs::read_dir(source_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("so") || ext.eq_ignore_ascii_case("dll") {
                    let file_name = match path.file_name() {
                        Some(name) => name,
                        None => continue, // Skip if filename is not valid
                    };
                    let dest_file = destination_path.join(file_name);

                    fs::copy(&path, &dest_file)?;
                }
            }
        }
    }

    Ok(())
}