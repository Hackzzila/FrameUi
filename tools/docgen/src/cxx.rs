use clang::*;

use std::collections::HashMap;

use super::{parse_comment, doc, c::*};

fn uppercase_first(s: &str) -> String {
  let mut c = s.chars();
  match c.next() {
    None => String::new(),
    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
  }
}

pub fn to_camel_case(s: &str) -> String {
  let mut i = 0;
  s.split("_").map(|x| {
    i += 1;
    if i != 1 {
      uppercase_first(x)
    } else {
      x.to_string()
    }
  }).collect::<String>()
}

pub fn to_pascal_case(s: &str) -> String {
  s.split("_").map(|x| {
    uppercase_first(x)
  }).collect::<String>()
}

impl Module<'_> {
  pub fn to_cxx(&self) -> String {
    let children = self.children.iter().map(|(_, x)| x.to_cxx()).collect::<Vec<_>>().join("\n");

    format!("
      namespace {} {{
        {}
      }}
    ", self.name, children)
  }
}

impl Definition<'_> {
  pub fn to_cxx(&self) -> String {
    match self {
      Self::Struct(s) => s.to_cxx(),
      Self::Typedef(s) => s.to_cxx(),
    }
  }
}

impl Typedef<'_> {
  pub fn to_cxx(&self) -> String {
    format!("{};", self.entity.get_pretty_printer().print())
  }
}

impl Struct<'_> {
  pub fn to_cxx(&self) -> String {
    let mut methods: Vec<_> = self.methods.iter().collect();
    methods.sort_by_key(|(_, x)| x.get_index());
    let methods = methods.iter().map(|(_, x)| x.to_cxx()).collect::<Vec<_>>().join("\n");

    format!("
      class {} {{
        public:
          {}

          ::{0} *GetInternalPointer() {{
            return self;
          }}

          ::{0} *TakeInternalPointer() {{
            ::{0} *out = self;
            self = nullptr;
            return out;
          }}

        private:
          ::{0} *self = nullptr;
      }};
    ", self.name, methods)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MethodKind {
  Constructor,
  Destructor,
  Method,
  StaticMethod,
}

impl Method<'_> {
  pub fn to_cxx(&self) -> String {
    let mut kind = match self.name.as_str() {
      "new" => MethodKind::Constructor,
      "drop" => MethodKind::Destructor,
      _ => MethodKind::StaticMethod,
    };

    let name = to_pascal_case(&self.name);
    let return_type = self.entity.get_result_type().unwrap().get_display_name();
    let c_name = self.entity.get_name().unwrap();

    let mut iter = c_name.splitn(2, "_");
    let struct_name = iter.next().unwrap().to_string();

    let mut args = Vec::new();
    let mut c_args = Vec::new();
    for arg in self.entity.get_arguments().unwrap() {
      let name = arg.get_display_name().unwrap();
      let ty = arg.get_type().unwrap().get_display_name();
      if name == "self" && kind == MethodKind::StaticMethod {
        c_args.push(name);
        kind = MethodKind::Method;
        continue
      }

      args.push(format!("{} {}", ty, name));
      c_args.push(name);
    }

    let args = args.join(", ");
    let c_args = c_args.join(", ");

    match kind {
      MethodKind::Constructor => {
        format!("
          {struct_name}({args}) {{
            self = {c_name}({c_args});
          }}",
          struct_name=struct_name,
          args=args,
          c_name=c_name,
          c_args=c_args,
        )
      }

      MethodKind::Destructor => {
        format!("
          ~{struct_name}() {{
            if (self) {{
              {c_name}(self);
            }}
          }}",
          struct_name=struct_name,
          c_name=c_name,
        )
      }

      _ => {
        format!("
          {static_keyword} {ret} {name}({args}) {{
            assert(self != nullptr);
            return {c_name}({c_args});
          }}",
          static_keyword=if kind == MethodKind::StaticMethod { "static" } else { "" },
          ret=return_type,
          name=name,
          args=args,
          c_name=c_name,
          c_args=c_args,
        )
      }
    }
  }
}