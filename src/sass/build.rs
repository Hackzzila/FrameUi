use std::{env, path::PathBuf};

#[cfg(target_env = "msvc")]
fn compile() {
  let target = std::env::var("TARGET").unwrap();
  let msvc_platform = if target.contains("x86_64") { "Win64" } else { "Win32" };

  let msbuild = cc::windows_registry::find_tool(&target, "msbuild.exe").expect("Failed to find MSBuild");
  let status = msbuild
    .to_command()
    .args(&[
      "win\\libsass.sln",
      "/p:LIBSASS_STATIC_LIB=1",
      "/p:Configuration=Release",
      "/p:WholeProgramOptimization=false",
      &format!("/p:Platform={}", msvc_platform),
    ])
    .current_dir("libsass")
    .status()
    .expect("Failed to run MSBuild");

  if !status.success() {
    panic!("Failed to build libsass");
  }

  println!(
    "cargo:rustc-link-search={}/libsass/win/bin",
    std::env::current_dir().unwrap().display()
  );
  println!("cargo:rustc-link-lib=static=libsass");
}

#[cfg(not(target_env = "msvc"))]
fn compile() {
  use std::{collections::HashMap, process::Command};

  let mut envs = HashMap::new();
  let make_executable = if cfg!(windows) {
    let tool = cc::Build::new().get_compiler();

    envs.insert("CC", tool.path().display().to_string());

    "mingw32-make"
  } else {
    "make"
  };

  let status = Command::new(make_executable)
    .envs(envs)
    .current_dir("libsass")
    .status()
    .expect("Failed to run make");

  if !status.success() {
    panic!("Failed to build libsass");
  }

  println!(
    "cargo:rustc-link-search={}/libsass/lib",
    std::env::current_dir().unwrap().display()
  );
  println!("cargo:rustc-link-lib=static=sass");
  println!("cargo:rustc-link-lib=c++");
}

fn main() {
  compile();

  let bindings = bindgen::Builder::default()
    .header("libsass/include/sass.h")
    .clang_arg("-Ilibsass/include")
    .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: false })
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .generate()
    .expect("Unable to generate bindings");

  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("Couldn't write bindings!");
}
