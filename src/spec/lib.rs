use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote, TokenStreamExt};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::{fs::File, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Spec {
  types: Vec<Type>,
  elements: Vec<Element>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Type {
  name: String,
  description: String,
  attrs: Option<Vec<Attribute>>,
  since: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Element {
  name: String,
  #[serde(rename = "type")]
  ty: String,
  description: String,
  children: Option<Vec<String>>,
  attrs: Option<Vec<Attribute>>,
  since: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum AttributeType {
  String,
  Number,
  Bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Attribute {
  name: String,
  #[serde(rename = "type")]
  ty: AttributeType,
  default: Value,
  description: String,
  since: String,
}

fn uppercase_first(s: &str) -> String {
  let mut c = s.chars();
  match c.next() {
    None => String::new(),
    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
  }
}

fn build_struct(name: &str, is_type: bool, attrs: &Vec<Attribute>, base_ident: Ident) -> TokenStream2 {
  let mut fields = Vec::new();
  let mut parse_fields = Vec::new();
  let mut fns = Vec::new();
  let mut defaults = Vec::new();
  for attr in attrs {
    let description = &attr.description;
    let name = format!("Name: {}", attr.name);
    let since = format!("Since: {}", attr.since);
    let docs = quote!(
      #[doc = #description]
      #[doc = ""]
      #[doc = #name]
      #[doc = ""]
      #[doc = #since]
    );

    let ident = format_ident!("{}", attr.name);

    let rust_ty = match attr.ty {
      AttributeType::String => quote!(String),
      AttributeType::Number => quote!(f64),
      AttributeType::Bool => quote!(bool),
    };

    fields.push(quote!(#docs pub #ident: #rust_ty));

    let get_ident = format_ident!("get_{}", ident);
    let set_ident = format_ident!("set_{}", ident);

    fns.push(quote!(
      #docs
      pub fn #get_ident(&self) -> #rust_ty {
        self.#ident.clone()
      }

      #docs
      pub fn #set_ident(&mut self, value: #rust_ty) {
        self.make_dirty();
        self.#ident = value;
      }
    ));

    let name = &attr.name;
    parse_fields.push(quote!(#name => {
      self.#set_ident(value.parse().unwrap());
      true
    }));

    let default = match &attr.default {
      Value::Bool(x) => quote!(#x),
      Value::String(x) => quote!(#x.to_string()),
      Value::Number(x) => {
        let f = x.as_f64().unwrap();
        quote!(#f)
      }

      _ => unimplemented!(),
    };

    defaults.push(quote!(#ident : #default));
  }

  let struct_ident = if is_type {
    format_ident!("{}TypeElementAttributes", uppercase_first(&name))
  } else {
    format_ident!("{}ElementAttributes", uppercase_first(&name))
  };

  quote!(
    #[derive(Debug)]
    pub struct #struct_ident {
      _base: #base_ident,
      #(#fields),*
    }

    impl #struct_ident {
      #(#fns)*

      pub fn parse(&mut self, key: &str, value: &str) -> bool {
        match key {
          #(#parse_fields),*
          _ => self._base.parse(key, value),
        }
      }
    }

    impl ::std::ops::Deref for #struct_ident {
      type Target = #base_ident;

      fn deref(&self) -> &Self::Target {
        &self._base
      }
    }

    impl ::std::ops::DerefMut for #struct_ident {
      fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._base
      }
    }

    impl ::std::default::Default for #struct_ident {
      fn default() -> Self {
        Self {
          _base: ::std::default::Default::default(),
          #(#defaults),*
        }
      }
    }
  )
}

use std::sync::Once;

static mut SPEC: Option<Spec> = None;
static SPEC_INIT: Once = Once::new();

fn get_spec() -> &'static Spec {
  unsafe {
    SPEC_INIT.call_once(|| {
      let f = File::open(Path::new(file!()).join("../spec.yml").canonicalize().unwrap()).unwrap();
      SPEC = serde_yaml::from_reader(f).unwrap();
    });
    SPEC.as_ref().unwrap()
  }
}

#[proc_macro]
pub fn generate(_: TokenStream) -> TokenStream {
  let mut tokens = TokenStream2::new();

  tokens.append_all(quote!(
    #[derive(Debug)]
    pub struct BaseElementAttributes {
      _dirty: bool,
    }

    impl BaseElementAttributes {
      pub fn parse(&mut self, key: &str, value: &str) -> bool {
        false
      }

      pub fn is_dirty(&self) -> bool {
        self._dirty
      }

      pub fn make_dirty(&mut self) {
        self._dirty = true;
      }
    }

    impl ::std::default::Default for BaseElementAttributes {
      fn default() -> Self {
        Self {
          _dirty: true
        }
      }
    }
  ));

  let path = Path::new(file!())
    .join("../spec.yml")
    .canonicalize()
    .unwrap()
    .display()
    .to_string();

  tokens.append_all(quote!(
    const _SPEC_YAML: &str = include_str!(#path);
  ));

  let spec = get_spec();

  let mut variants = Vec::new();
  let mut name_match = Vec::new();
  for ty in &spec.types {
    let ident = format_ident!("{}", uppercase_first(&ty.name));
    let description = &ty.description;
    let name = format!("Name: {}", ty.name);
    let since = format!("Since: {}", ty.since);
    variants.push(quote!(
      #[doc = #description]
      #[doc = ""]
      #[doc = #name]
      #[doc = ""]
      #[doc = #since]
      #ident
    ));

    let name = &ty.name;
    name_match.push(quote!(
      Self::#ident => write!(f, #name)
    ));

    tokens.append_all(build_struct(
      &ty.name,
      true,
      ty.attrs.as_ref().unwrap_or(&Vec::new()),
      format_ident!("BaseElementAttributes"),
    ));
  }

  tokens.append_all(quote!(
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub enum ElementType {
      #(#variants),*
    }

    impl ::std::fmt::Display for ElementType {
      fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
          #(#name_match),*
        }
      }
    }
  ));

  let mut variants = Vec::new();
  let mut name_match = Vec::new();
  let mut type_match = Vec::new();
  let mut children_match = Vec::new();
  for el in &spec.elements {
    let ident = format_ident!("{}", el.name);
    let struct_ident = format_ident!("{}Element", uppercase_first(&el.name));
    let description = &el.description;
    let name = format!("Name: {}", el.name);
    let ty = format!("Type: {}", el.ty);
    let children = format!("Children: {}", el.children.as_ref().unwrap_or(&Vec::new()).join(", "));
    let since = format!("Since: {}", el.since);
    variants.push(quote!(
      #[doc = #description]
      #[doc = ""]
      #[doc = #name]
      #[doc = ""]
      #[doc = #ty]
      #[doc = ""]
      #[doc = #children]
      #[doc = ""]
      #[doc = #since]
      #ident(#struct_ident)
    ));

    let name = &el.name;
    name_match.push(quote!(
      Self::#ident(..) => #name
    ));

    let type_ident = format_ident!("{}", uppercase_first(&el.ty));

    type_match.push(quote!(
      Self::#ident(..) => ElementType::#type_ident
    ));

    let mut children = Vec::new();
    if let Some(children_vec) = &el.children {
      children.push(quote!(ElementType::Comment));
      for child in children_vec {
        let ident = format_ident!("{}", uppercase_first(&child));
        children.push(quote!(ElementType::#ident));
      }
    }

    children_match.push(quote!(
      Self::#ident(..) => &[#(#children),*]
    ));

    let base_ident = format_ident!("{}TypeElementAttributes", uppercase_first(&el.ty));
    tokens.append_all(build_struct(
      &el.name,
      false,
      el.attrs.as_ref().unwrap_or(&Vec::new()),
      base_ident,
    ));
  }

  tokens.append_all(quote!(
    #[derive(Debug)]
    pub enum Element {
      Root,
      Text(String),
      Comment(String),
      Error,
      #(#variants),*
    }

    impl Element {
      pub fn is_error(&self) -> bool {
        match self {
          Self::Error => true,
          _ => false,
        }
      }

      pub fn get_name(&self) -> &str {
        match self {
          Self::Root => "#root",
          Self::Text(..) => "#text",
          Self::Comment(..) => "#comment",
          Self::Error => "#error",
          #(#name_match),*
        }
      }

      pub fn get_type(&self) -> ElementType {
        match self {
          Self::Root => ElementType::Base,
          Self::Text(..) => ElementType::Text,
          Self::Comment(..) => ElementType::Comment,
          Self::Error => ElementType::Base, // dummy value
          #(#type_match),*
        }
      }

      pub fn get_children(&self) -> &[ElementType] {
        match self {
          Self::Root => &[ElementType::Base, ElementType::Comment],
          Self::Text(..) => &[],
          Self::Comment(..) => &[],
          Self::Error => &[],
          #(#children_match),*
        }
      }
    }
  ));

  tokens.into()
}

use quote::ToTokens;
use syn::{parse_macro_input, parse_quote, ItemStruct};

#[proc_macro_attribute]
pub fn element(_: TokenStream, item: TokenStream) -> TokenStream {
  let mut tokens = TokenStream2::new();
  let mut item = parse_macro_input!(item as ItemStruct);

  let name = item.ident.to_string();
  let name = name.trim_end_matches("Element");

  if name == "Base" {
    tokens.append_all(item.to_token_stream());
    return tokens.into();
  }

  let mut base_ident = format_ident!("BaseElement");
  if !name.ends_with("Type") {
    let spec = get_spec();

    let el = spec.elements.iter().find(|x| x.name == name).unwrap();

    base_ident = format_ident!("{}TypeElement", uppercase_first(&el.ty));

    if let syn::Fields::Named(fields) = &mut item.fields {
      let attrs_ident = format_ident!("{}Attributes", item.ident);
      let ident = format_ident!("attrs");
      fields.named.push(syn::Field {
        attrs: Vec::new(),
        vis: parse_quote!(pub),
        ident: Some(ident),
        colon_token: Some(parse_quote!(:)),
        ty: parse_quote!(#attrs_ident),
      });
    } else {
      unimplemented!();
    }
  }

  if let syn::Fields::Named(fields) = &mut item.fields {
    let base = format_ident!("base");
    fields.named.push(syn::Field {
      attrs: Vec::new(),
      vis: syn::Visibility::Inherited,
      ident: Some(base),
      colon_token: Some(parse_quote!(:)),
      ty: parse_quote!(#base_ident),
    });

    let ident = &item.ident;
    tokens.append_all(quote!(
      impl ::std::ops::Deref for #ident {
        type Target = #base_ident;

        fn deref(&self) -> &Self::Target {
          &self.base
        }
      }

      impl ::std::ops::DerefMut for #ident {
        fn deref_mut(&mut self) -> &mut Self::Target {
          &mut self.base
        }
      }
    ));
  } else {
    unimplemented!();
  }

  tokens.append_all(item.to_token_stream());
  return tokens.into();
}

#[proc_macro]
pub fn parse_element(_: TokenStream) -> TokenStream {
  let spec = get_spec();

  let mut matches = Vec::new();
  for el in &spec.elements {
    let name = &el.name;
    let variant_ident = format_ident!("{}", el.name);
    let struct_ident = format_ident!("{}Element", el.name);

    matches.push(quote!(
      #name => {
        let mut el = ::dom::#struct_ident::default();

        for attr in e.attributes() {
          let attr = attr.map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?;
          let key = reader.decode(attr.key).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?;
          let value = attr.unescaped_value().map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?;
          let value = reader.decode(&value).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?;
          if !el.attrs.parse(key, value) {
            // return Err(Error::InvalidAttribute(key.to_string(), name.to_string()));
            // printer.add(Diag::InvalidAttribute(key.to_string(), name.to_string()), reader.buffer_position(), file_id);
            reporter.add_diagnostic(Diagnostic {
              pos: reader.buffer_position(),
              file_id: file_id.clone(),
              data: DiagnosticData::InvalidAttribute(
                key.to_string(),
                name.to_string(),
              ),
            })?;
          }
        }

        ::dom::Element::#variant_ident(el)
      }
    ))
  }

  quote!(
    {
      match name {
        #(#matches),*
        // _ => return Err(Error::InvalidElement(name.to_string())),
        _ => {
          // printer.add(Diag::InvalidElement(name.to_string()), reader.buffer_position(), file_id);
          reporter.add_diagnostic(Diagnostic {
            pos: reader.buffer_position(),
            file_id: file_id.clone(),
            data: DiagnosticData::InvalidElement(name.to_string()),
          })?;
          ::dom::Element::Error
        }
      }
    }
  )
  .into()
}
