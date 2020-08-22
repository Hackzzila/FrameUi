use clang::*;

use std::collections::BTreeMap;

use super::{doc, parse_comment};

#[derive(Debug, Clone)]
pub struct Module<'tu> {
  pub name: String,
  pub children: BTreeMap<String, Definition<'tu>>,
}

impl Module<'_> {
  pub fn to_docs(&self) -> doc::Module {
    let mut children: Vec<_> = self.children.iter().map(|(_, x)| x.to_docs()).collect();
    children.sort_by_key(|x| match x {
      doc::Definition::Struct(x) => x.name.clone(),
      doc::Definition::Typedef(x) => x.name.clone(),
      doc::Definition::DataStruct(x) => x.name.clone(),
    });

    doc::Module {
      name: self.name.clone(),
      children,
    }
  }
}

#[derive(Debug, Clone)]
pub enum Definition<'tu> {
  Struct(Struct<'tu>),
  Typedef(Typedef<'tu>),
  DataStruct(DataStruct<'tu>),
}

impl Definition<'_> {
  pub fn to_docs(&self) -> doc::Definition {
    match self {
      Self::Struct(s) => doc::Definition::Struct(s.to_docs()),
      Self::Typedef(s) => doc::Definition::Typedef(s.to_docs()),
      Self::DataStruct(s) => doc::Definition::DataStruct(s.to_docs()),
    }
  }
}

#[derive(Debug, Clone)]
pub struct Typedef<'tu> {
  pub name: String,
  pub entity: Entity<'tu>,
}

impl Typedef<'_> {
  pub fn to_docs(&self) -> doc::Typedef {
    let (description, _) = parse_comment(self.entity.get_comment().unwrap());

    let description = if description.len() == 0 {
      None
    } else {
      Some(description)
    };

    doc::Typedef {
      name: self.name.clone(),
      declaration: self.entity.get_pretty_printer().print(),
      brief: self.entity.get_comment_brief(),
      description,
    }
  }
}

#[derive(Debug, Clone)]
pub struct Struct<'tu> {
  pub name: String,
  pub module: String,
  pub methods: BTreeMap<String, Method<'tu>>,
  pub entity: Entity<'tu>,
}

impl Struct<'_> {
  pub fn to_docs(&self) -> doc::Struct {
    let mut methods: Vec<_> = self.methods.iter().collect();
    methods.sort_by_key(|(_, x)| x.get_index());
    let methods = methods.iter().map(|(_, x)| x.to_docs()).collect();

    let (description, _) = parse_comment(self.entity.get_comment().unwrap());

    let description = if description.len() == 0 {
      None
    } else {
      Some(description)
    };

    doc::Struct {
      name: self.name.clone(),
      brief: self.entity.get_comment_brief(),
      description,
      methods,
    }
  }
}

#[derive(Debug, Clone)]
pub struct Method<'tu> {
  pub name: String,
  pub entity: Entity<'tu>,
}

impl Method<'_> {
  pub fn get_index(&self) -> usize {
    let comment = self.entity.get_comment().unwrap();
    let (_, config) = parse_comment(comment);
    config["index"].as_ref().unwrap().parse().unwrap()
  }

  pub fn to_docs(&self) -> doc::Method {
    let mut declaration = self.entity.get_pretty_printer().print();
    if declaration.len() > 80 {
      let mut out = String::new();

      let indent_len = declaration.find('(').unwrap() + 1;
      let indent = " ".repeat(indent_len);

      let mut segments = declaration.split(',').peekable();
      out += segments.next().unwrap();
      out += ",\n";

      while let Some(segment) = segments.next() {
        out += &format!("{}{}", indent, segment.trim());
        if segments.peek().is_some() {
          out += ",\n";
        }
      }

      declaration = out;
    }

    let comment = self.entity.get_comment().unwrap();
    let (description, _) = parse_comment(comment);

    let description = if description.len() == 0 {
      None
    } else {
      Some(description)
    };

    doc::Method {
      name: self.name.clone(),
      declaration,
      brief: self.entity.get_comment_brief(),
      description,
    }
  }
}

#[derive(Debug, Clone)]
pub struct DataStruct<'tu> {
  pub name: String,
  pub module: String,
  pub fields: Vec<Field<'tu>>,
  pub entity: Entity<'tu>,
}

impl DataStruct<'_> {
  pub fn to_docs(&self) -> doc::DataStruct {
    let fields = self.fields.iter().map(|x| x.to_docs()).collect();

    let (description, _) = parse_comment(self.entity.get_comment().unwrap());

    let description = if description.len() == 0 {
      None
    } else {
      Some(description)
    };

    doc::DataStruct {
      name: self.name.clone(),
      brief: self.entity.get_comment_brief(),
      description,
      fields,
    }
  }
}

#[derive(Debug, Clone)]
pub struct Field<'tu> {
  pub entity: Entity<'tu>,
}

impl Field<'_> {
  pub fn to_docs(&self) -> doc::Field {
    doc::Field {
      name: self.entity.get_name().unwrap(),
      declaration: self.entity.get_pretty_printer().print(),
      brief: self.entity.get_comment_brief(),
      description: self.entity.get_comment(),
    }
  }
}
