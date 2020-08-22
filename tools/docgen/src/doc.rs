use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct Root {
  pub language: String,
  pub keywords: HashMap<String, String>,
  pub modules: Vec<Module>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Module {
  pub name: String,
  pub children: Vec<Definition>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Definition {
  Typedef(Typedef),
  Struct(Struct),
  DataStruct(DataStruct),
}

#[derive(Debug, Clone, Serialize)]
pub struct Typedef {
  pub name: String,
  pub declaration: String,
  pub brief: Option<String>,
  pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Struct {
  pub name: String,
  pub brief: Option<String>,
  pub description: Option<String>,
  pub methods: Vec<Method>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Method {
  pub name: String,
  pub declaration: String,
  pub brief: Option<String>,
  pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DataStruct {
  pub name: String,
  pub brief: Option<String>,
  pub description: Option<String>,
  pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Field {
  pub name: String,
  pub declaration: String,
  pub brief: Option<String>,
  pub description: Option<String>,
}
