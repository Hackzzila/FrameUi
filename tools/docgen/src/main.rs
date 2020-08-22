use clang::*;

use std::collections::{BTreeMap, HashMap};

fn parse_comment(comment: String) -> (String, HashMap<String, Option<String>>) {
  let mut description = String::new();
  let mut config = HashMap::new();

  for line in comment.lines().skip(1) {
    let line = line.trim_start_matches(" *");
    if line.is_empty() {
      description += "\n";
    } else if line == "/" {
      continue;
    } else if line.starts_with(' ') {
      description += line;
      description += "\n";
    } else {
      let properties = line.split(',');
      for prop in properties {
        let prop = prop.trim_start_matches(" *");
        let mut split = prop.split('=');
        let name = split.next().unwrap();
        let value = split.next();
        config.insert(name.to_string(), value.map_or(None, |x| Some(x.to_string())));
      }
    }
  }

  (description, config)
}

mod c;
mod cxx;
mod doc;

fn main() {
  let clang = Clang::new().unwrap();
  let index = Index::new(&clang, false, false);
  let tu = index.parser("./include/project-a.h").parse().unwrap();

  let mut modules = BTreeMap::new();

  let structs = tu
    .get_entity()
    .get_children()
    .into_iter()
    .filter(|e| e.get_kind() == EntityKind::TypedefDecl && !e.is_in_system_header())
    .collect::<Vec<_>>();

  for e in structs {
    let (_, config) = parse_comment(e.get_comment().unwrap_or_default());
    if let Some(module) = config.get("module") {
      let module = module.clone().unwrap();

      let entry = modules.entry(module.clone()).or_insert(c::Module {
        name: module.clone(),
        children: BTreeMap::new(),
      });

      let name = e.get_name().unwrap();

      match e.get_type().unwrap().get_canonical_type().get_kind() {
        TypeKind::Record => {
          let fields = e.get_type().unwrap().get_canonical_type().get_fields().unwrap();

          if !fields.is_empty() {
            entry.children.insert(
              name.clone(),
              c::Definition::DataStruct(c::DataStruct {
                name,
                module,
                entity: e,
                fields: fields.into_iter().map(|x| c::Field { entity: x }).collect(),
              }),
            );
          } else {
            entry.children.insert(
              name.clone(),
              c::Definition::Struct(c::Struct {
                name,
                module,
                methods: BTreeMap::new(),
                entity: e,
              }),
            );
          }
        }

        _ => {
          entry
            .children
            .insert(name.clone(), c::Definition::Typedef(c::Typedef { name, entity: e }));
        }
      }
    }
  }

  let fns = tu
    .get_entity()
    .get_children()
    .into_iter()
    .filter(|e| e.get_kind() == EntityKind::FunctionDecl && !e.is_in_system_header())
    .collect::<Vec<_>>();

  for e in fns {
    let (_, config) = parse_comment(e.get_comment().unwrap_or_default());
    if let Some(module) = config.get("module") {
      let name = e.get_name().unwrap();
      let is_struct = name.chars().next().unwrap().is_ascii_uppercase();

      if is_struct {
        let mut iter = name.splitn(2, "_");
        let struct_name = iter.next().unwrap().to_string();
        let method_name = iter.next().unwrap().to_string();

        let entry = modules
          .get_mut(module.as_ref().unwrap())
          .unwrap()
          .children
          .get_mut(&struct_name)
          .unwrap();

        if let c::Definition::Struct(entry) = entry {
          entry.methods.insert(
            method_name.clone(),
            c::Method {
              name: method_name,
              entity: e,
            },
          );
        }
      }
    }
  }

  // {
  //   let cxx: Vec<_> = modules.iter().map(|(_, x)| x.to_cxx()).collect();

  //   let mut f = std::fs::File::create("./include/project-a.hpp").unwrap();
  //   f.write_all(
  //     b"#include <cassert>

  //     namespace frameui {

  //     namespace c_api {
  //       #include \"project-a.h\"
  //     }",
  //   )
  //   .unwrap();

  //   for module in cxx {
  //     f.write_all(module.as_bytes()).unwrap();
  //   }

  //   f.write_all(b"}").unwrap();
  // }

  let _ = std::process::Command::new("clang-format")
    .args(&["--style=google", "-i", "include/project-a.hpp"])
    .status();

  let mut modules: Vec<_> = modules.iter().map(|(_, x)| x.to_docs()).collect();
  modules.sort_by_key(|x| x.name.clone());

  let mut keywords = HashMap::new();
  keywords.insert("Struct".to_string(), "struct".to_string());
  keywords.insert("Typedef".to_string(), "typedef".to_string());
  keywords.insert("DataStruct".to_string(), "struct".to_string());

  let root = doc::Root {
    language: "C".to_string(),
    modules,
    keywords,
  };

  let json = serde_json::to_string(&root).unwrap();

  // println!("{}", json);

  // println!("{:#?}", modules["event"].to_docs());

  // let docs = modules["event"]["EventHandler"].to_docs();
  // let json = serde_json::to_string(&docs).unwrap();

  use std::io::prelude::*;
  let mut f = std::fs::File::create("./doc/src/doc.json").unwrap();
  f.write_all(json.as_bytes()).unwrap();
}
