use std::env;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use quote::{quote, format_ident};
use serde::{Serialize, Deserialize};
use proc_macro2::{TokenStream, Ident};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Protocol {
  version: ProtocolVersion,
  domains: Vec<ProtocolDomain>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProtocolVersion {
  major: String,
  minor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProtocolDomain {
  domain: String,
  description: Option<String>,
  dependencies: Option<Vec<String>>,
  types: Option<Vec<DomainType>>,
  commands: Option<Vec<Command>>,
  events: Option<Vec<Event>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Event {
  name: String,
  parameters: Option<Vec<PropertyType>>,
  description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Command {
  #[serde(flatten)]
  event: Event,

  returns: Option<Vec<PropertyType>>,
  r#async: Option<bool>,
  redirect: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArrayType {
  items: RefTypeOr<ArrayItemType>,
  min_items: Option<f64>,
  max_items: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
enum ArrayItemType {
  Number,
  Integer,
  Boolean,
  String(StringType),
  Any,
  Object(ObjectType),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ObjectType {
  properties: Option<Vec<PropertyType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StringType {
  r#enum: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RefType {
  #[serde(rename = "$ref")]
  r#ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum RefTypeOr<T> {
  Ref(RefType),
  Other(T),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
enum PrimitiveType {
  Number,
  Integer,
  Boolean,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PropertyType {
  name: String,
  optional: Option<bool>,
  description: Option<String>,

  #[serde(flatten)]
  data: RefTypeOr<ProtocolType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
enum ProtocolType {
  String(StringType),
  Object(ObjectType),
  Array(ArrayType),
  Number,
  Integer,
  Boolean,
  Any,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DomainType {
  id: String,
  description: Option<String>,

  #[serde(flatten)]
  data: DomainTypeData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
enum DomainTypeData {
  String(StringType),
  Object(ObjectType),
  Array(ArrayType),
  Number,
  Integer,
  Boolean,
}

fn array_type_to_inner(items: RefTypeOr<ArrayItemType>) -> TokenStream {
  match items {
    RefTypeOr::Ref(reference) => ref_to_type(reference),

    RefTypeOr::Other(other) => match other {
      ArrayItemType::Number => quote!(f64),
      ArrayItemType::Integer => quote!(i64),
      ArrayItemType::Boolean => quote!(bool),
      ArrayItemType::Any => quote!(serde_json::Value),
      ArrayItemType::String(s) => {
        if s.r#enum.is_some() {
          unimplemented!();
        }

        quote!(String)
      },

      ArrayItemType::Object(obj) => {
        if obj.properties.is_some() {
          unimplemented!();
        }

        quote!(std::collections::HashMap<String, serde_json::Value>)
      },
    }
  }
}

fn ref_to_type(reference: RefType) -> TokenStream {
  let name = reference.r#ref;
  if name.contains(".") {
    let mut split = name.split(".");
    let ns = split.next().unwrap();
    let name = split.next().unwrap();

    let ns = format_ident!("{}", ns.to_lowercase());
    let ident = format_ident!("{}", name);
    quote!(Box<super::#ns::#ident>)
  } else {
    let ident = format_ident!("{}", name);
    quote!(Box<#ident>)
  }
}

fn generate_enum(ident: &Ident, description: String, variants: Vec<String>) -> TokenStream {
  let variants = variants.iter().map(|x| {
    if x.contains("-") {
      let ident = format_ident!("r#{}", x.replace("-", "_"));
      quote!(
        #[serde(rename = #x)]
        #ident
      )
    } else {
      let ident = format_ident!("r#{}", x);
      quote!(#ident)
    }
  });


  quote!(
    #[doc = #description]
    #[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
    pub enum #ident {
      #(#variants),*
    }
  )
}

fn generate_properties(ident: &Ident, types: &mut Vec<TokenStream>, props: Vec<PropertyType>, public: bool) -> Vec<TokenStream> {
  props.into_iter().map(|prop| {
    let ty = match prop.data {
      RefTypeOr::Ref(reference) => {
        // if reference.r#ref == ident.to_string() {
        //   quote!(Box<Self>)
        // } else {
        //   ref_to_type(reference)
        // }
        ref_to_type(reference)
      },

      RefTypeOr::Other(other) => match other {
        ProtocolType::Number => quote!(f64),
        ProtocolType::Integer => quote!(i64),
        ProtocolType::Boolean => quote!(bool),
        ProtocolType::Any => quote!(serde_json::Value),
        ProtocolType::String(s) => {
          if let Some(variants) = s.r#enum {
            let ident = format_ident!("{}{}", ident, uppercase_first(&prop.name));
            types.push(generate_enum(&ident, prop.description.clone().unwrap_or_default(), variants));
            quote!(#ident)
          } else {
            quote!(String)
          }
        },

        ProtocolType::Array(arr) => {
          let inner = array_type_to_inner(arr.items);
          quote!(Vec<#inner>)
        },

        ProtocolType::Object(obj) => {
          if obj.properties.is_some() {
            unimplemented!();
          }

          quote!(std::collections::HashMap<String, serde_json::Value>)
        }
      }
    };

    let (def, ty) = if prop.optional.unwrap_or_default() {
      (quote!(#[serde(default)]), quote!(Option<#ty>))
    } else {
      (quote!(), ty)
    };

    let name = prop.name;
    let ident = format_ident!("r#{}", inflector::cases::snakecase::to_snake_case(&name));
    let description = prop.description.unwrap_or_default();

    let vis = if public { quote!(pub) } else { quote!() };

    quote!(
      #[doc = #description]
      #[serde(rename = #name)]
      #def
      #vis #ident: #ty
    )
  }).collect()
}

fn uppercase_first(s: &str) -> String {
  let mut c = s.chars();
  match c.next() {
    None => String::new(),
    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
  }
}

fn main() {
  let mut browser: Protocol = serde_json::from_reader(File::open("devtools-protocol/json/browser_protocol.json").unwrap()).unwrap();
  let js: Protocol = serde_json::from_reader(File::open("devtools-protocol/json/js_protocol.json").unwrap()).unwrap();

  browser.domains.extend(js.domains);

  let mut domains = Vec::new();
  let mut domain_names = Vec::new();

  for domain in browser.domains {
    domain_names.push(domain.domain.clone());

    let mut types = Vec::new();
    let mut commands = Vec::new();
    let mut command_results = Vec::new();

    for ty in domain.types.unwrap_or_default() {
      let ident = format_ident!("{}", ty.id);
      let description = ty.description.unwrap_or_default();

      match ty.data {
        DomainTypeData::Integer => {
          types.push(quote!(
            #[doc = #description]
            pub type #ident = i64;
          ));
        },

        DomainTypeData::Number => {
          types.push(quote!(
            #[doc = #description]
            pub type #ident = f64;
          ));
        },

        DomainTypeData::Boolean => {
          types.push(quote!(
            #[doc = #description]
            pub type #ident = bool;
          ));
        },

        DomainTypeData::String(s) => {
          if let Some(variants) = s.r#enum {
            types.push(generate_enum(&ident, description, variants));
          } else {
            types.push(quote!(
              #[doc = #description]
              pub type #ident = String;
            ));
          }
        }

        DomainTypeData::Array(arr) => {
          let ty = array_type_to_inner(arr.items);

          types.push(quote!(
            #[doc = #description]
            pub type #ident = Vec<#ty>;
          ));
        }

        DomainTypeData::Object(obj) => {
          if let Some(properties) = obj.properties {
            let properties: Vec<_> = generate_properties(&ident, &mut types, properties, true);

            types.push(quote!(
              #[doc = #description]
              #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
              pub struct #ident {
                #(#properties),*
              }
            ));
          } else {
            types.push(quote!(
              #[doc = #description]
              pub type #ident = std::collections::HashMap<String, serde_json::Value>;
            ));
          }
        }
      }
    }

    for command in domain.commands.unwrap_or_default() {
      let name = format!("{}.{}", domain.domain, command.event.name);
      let ident = format_ident!("{}", uppercase_first(&command.event.name));
      let description = command.event.description.unwrap_or_default();

      if let Some(parameters) = command.event.parameters {
        let all_optional = parameters.iter().all(|x| x.optional.unwrap_or_default());
        let props: Vec<_> = generate_properties(&ident, &mut types, parameters, true);

        types.push(quote!(
          #[doc = #description]
          #[derive(Debug, Clone, PartialEq, serde::Deserialize)]
          pub struct #ident {
            #(#props),*
          }
        ));

        let child = if all_optional {
          quote!(Option<#ident>)
        } else {
          quote!(#ident)
        };

        commands.push(quote!(
          #[serde(rename = #name)]
          #[doc = #description]
          #ident(#child)
        ));
      } else {
        commands.push(quote!(
          #[serde(rename = #name)]
          #[doc = #description]
          #ident(Option<serde_json::Value>)
        ));
      }

      if let Some(ret) = command.returns {
        let props: Vec<_> = generate_properties(&ident, &mut types, ret, false);

        command_results.push(quote!(
          #[doc = #description]
          #ident {
            #(#props),*
          }
        ));
      }
    }

    let ident = format_ident!("{}", domain.domain.to_lowercase());
    let description = domain.description.unwrap_or_default();
    let dependencies = format!("Depends on: {}", domain.dependencies.unwrap_or_default().join(", "));
    domains.push(quote!(
      #[doc = #description]
      #[doc = #dependencies]
      pub mod #ident {
        #(#types)*

        #[derive(Debug, Clone, PartialEq, serde::Deserialize)]
        #[serde(tag = "method", content = "params")]
        pub enum Command {
          #(#commands),*
        }

        #[derive(Debug, Clone, PartialEq, serde::Serialize)]
        #[serde(untagged)]
        pub enum CommandResult {
          #(#command_results),*
        }
      }
    ));
  }

  let command_variants = domain_names.iter().map(|name| {
    let variant_ident = format_ident!("{}", name);
    let mod_ident = format_ident!("{}", name.to_lowercase());
    quote!(#variant_ident(#mod_ident::Command))
  });

  let command_result_variants = domain_names.iter().map(|name| {
    let variant_ident = format_ident!("{}", name);
    let mod_ident = format_ident!("{}", name.to_lowercase());
    quote!(#variant_ident(#mod_ident::CommandResult))
  });

  let version = format!("DevTools Protocol Version {}.{}", browser.version.major, browser.version.minor);
  let bindings = quote!(
    #[doc = #version]
    #(#domains)*

    #[derive(Debug, Clone, PartialEq, serde::Deserialize)]
    pub struct Command {
      pub id: u64,
      #[serde(flatten)]
      pub data: CommandData,
    }

    #[derive(Debug, Clone, PartialEq, serde::Deserialize)]
    #[serde(untagged)]
    pub enum CommandData {
      #(#command_variants),*
    }

    #[derive(Debug, Clone, PartialEq, serde::Serialize)]
    pub struct CommandResult {
      pub id: u64,
      pub result: CommandResultData,
    }

    #[derive(Debug, Clone, PartialEq, serde::Serialize)]
    #[serde(untagged)]
    pub enum CommandResultData {
      #(#command_result_variants),*
    }
  );

  let path = Path::new(&env::var("OUT_DIR").unwrap()).join("bindings.rs");
  let mut out = File::create(&path).unwrap();
  out.write_all(bindings.to_string().as_bytes()).unwrap();

  let _ = std::process::Command::new("rustfmt")
    .arg(path)
    .status();
}