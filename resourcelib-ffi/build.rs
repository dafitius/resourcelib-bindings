// mod build;

use std::{env, fs, io};
use std::path::{Path, PathBuf};


macro_rules! warn {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    let lib_path = PathBuf::from(env::var("DEP_RESOURCELIB_SYS_LIB_PATH").unwrap());
    println!("cargo:lib_path={}", lib_path.display());
    // warn!("{:?}", lib_path);
}
