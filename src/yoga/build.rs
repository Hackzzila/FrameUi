use std::env;
use std::path::PathBuf;

#[derive(Debug)]
struct BindgenCallbacks;

impl bindgen::callbacks::ParseCallbacks for BindgenCallbacks {
  fn enum_variant_name(&self, enum_name: Option<&str>, name: &str, _: bindgen::callbacks::EnumVariantValue) -> Option<String> {
    let enum_name = enum_name?.trim_start_matches("enum").trim();
    name.strip_prefix(enum_name).map(|x| x.to_string())
  }

  fn include_file(&self, filename: &str) {
    println!("cargo:rerun-if-changed={}", filename);
  }
}

fn main() {
  println!("cargo:rerun-if-changed=yoga/yoga/Yoga.h");

  cc::Build::new()
    .file("yoga/yoga/event/event.cpp")
    .file("yoga/yoga/internal/experiments.cpp")
    .file("yoga/yoga/log.cpp")
    .file("yoga/yoga/Utils.cpp")
    .file("yoga/yoga/YGConfig.cpp")
    .file("yoga/yoga/YGEnums.cpp")
    .file("yoga/yoga/YGLayout.cpp")
    .file("yoga/yoga/YGNode.cpp")
    .file("yoga/yoga/YGNodePrint.cpp")
    .file("yoga/yoga/YGStyle.cpp")
    .file("yoga/yoga/YGValue.cpp")
    .file("yoga/yoga/Yoga.cpp")
    .flag_if_supported("-fno-omit-frame-pointer")
    .flag_if_supported("-fexceptions")
    .flag_if_supported("-fvisibility=hidden")
    .flag_if_supported("-ffunction-sections")
    .flag_if_supported("-fdata-sections")
    .flag_if_supported("-Wall")
    .flag_if_supported("-Werror")
    .flag_if_supported("-O2")
    .flag_if_supported("-std=c++11")
    .flag_if_supported("-DYG_ENABLE_EVENTS")
    .flag_if_supported("-fPIC")
    .cpp(true)
    .include("yoga")
    .define("DEBUG", None)
    .compile("yoga");

  let bindings = bindgen::Builder::default()
    .header("yoga/yoga/Yoga.h")
    .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: false })
    .whitelist_type("YG.*")
    .whitelist_var("YG.*")
    .whitelist_function("YG.*")
    .parse_callbacks(Box::new(BindgenCallbacks))
    .generate()
    .expect("Unable to generate bindings");

  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("Couldn't write bindings!");
}