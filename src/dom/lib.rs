use std::sync::RwLock;

use serde::{Deserialize, Serialize};

//                               [F]rame
//                                     [U]
//                                           [i]
//                                                 [S]tandard
//                                                       Version
pub const MAGIC_BYTES: &[u8] = &[0x46, 0x55, 0x69, 0x53, 0];

pub mod tree;
use tree::Node;

fn safe_yoga_node_new() -> yoga::Node {
  unsafe { yoga::Node::new() }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Element {
  pub data: ElementData,

  pub raw_attributes: RawElementAttributes,

  pub classes: Vec<String>,
  pub id: Option<String>,
  pub style: Vec<style::StyleRule>,

  #[serde(skip, default = "safe_yoga_node_new")]
  pub yg: yoga::Node,

  #[serde(skip)]
  pub computed: style::ComputedStyle,
}

impl PartialEq for Element {
  fn eq(&self, other: &Self) -> bool {
    // The yoga pointer for each element SHOULD be unique...
    self.yg == other.yg
  }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize)]
pub enum RawAttributeValue {
  Raw {
    value: String,

    #[serde(skip)]
    up_to_date: bool,
  },

  Script {
    script: String,

    #[serde(skip)]
    up_to_date: bool,

    #[serde(skip)]
    ast: Option<rhai::AST>,
  },
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RawElementAttributes {
  pub class: Option<RawAttributeValue>,
  pub id: Option<RawAttributeValue>,
  pub style: Option<RawAttributeValue>,
}

impl Element {
  #[must_use]
  pub fn new(data: ElementData, attrs: RawElementAttributes) -> Self {
    Self {
      data,

      raw_attributes: attrs,
      classes: Vec::new(),
      id: None,
      style: Vec::new(),

      yg: unsafe { yoga::Node::new() },
      computed: style::ComputedStyle::default(),
    }
  }

  pub fn prepare_yoga(&mut self) {
    unsafe {
      self.yg.set_width(self.computed.width);
      self.yg.set_height(self.computed.height);
      self.yg.set_margin(yoga::Edge::Top, self.computed.margin_top);
      self.yg.set_margin(yoga::Edge::Bottom, self.computed.margin_bottom);
      self.yg.set_margin(yoga::Edge::Left, self.computed.margin_left);
      self.yg.set_margin(yoga::Edge::Right, self.computed.margin_right);
    }
  }

  pub fn compute_attributes(&mut self, engine: &rhai::Engine, scope: &mut rhai::Scope) {
    if let Some(class) = &mut self.raw_attributes.class {
      match class {
        RawAttributeValue::Raw { value, up_to_date } => {
          if !*up_to_date {
            self.classes = value.split_ascii_whitespace().map(|s| s.to_string()).collect()
          }
        }

        RawAttributeValue::Script {
          script,
          up_to_date,
          ast,
        } => {
          if ast.is_none() || !*up_to_date {
            *ast = Some(engine.compile_expression_with_scope(scope, script).unwrap());
            *up_to_date = true;
          }

          let dynamic_classes: Vec<rhai::Dynamic> = engine.eval_ast_with_scope(scope, ast.as_ref().unwrap()).unwrap();
          self.classes.clear();
          for class in dynamic_classes {
            self.classes.push(class.take_string().unwrap());
          }
        }
      }
    } else {
      self.classes.clear();
    }

    if let Some(id) = &mut self.raw_attributes.id {
      match id {
        RawAttributeValue::Raw { value, up_to_date } => {
          if !*up_to_date {
            self.id = Some(value.clone());
          }
        }

        RawAttributeValue::Script {
          script,
          up_to_date,
          ast,
        } => {
          if ast.is_none() || !*up_to_date {
            *ast = Some(engine.compile_expression_with_scope(scope, script).unwrap());
            *up_to_date = true;
          }

          self.id = Some(engine.eval_ast_with_scope(scope, ast.as_ref().unwrap()).unwrap());
        }
      }
    } else {
      self.id = None;
    }
  }

  #[must_use]
  pub fn get_render(&self) -> style::RenderStyle {
    unsafe {
      style::RenderStyle {
        width: self.yg.get_width(),
        height: self.yg.get_height(),
        top: self.yg.get_top(),
        left: self.yg.get_left(),
        background_color: self.computed.background_color,
      }
    }
  }

  #[must_use]
  pub fn get_local_name(&self) -> &str {
    match self.data {
      ElementData::Root(..) => "#root",
      ElementData::Unstyled(..) => "Unstyled",
    }
  }

  #[must_use]
  pub fn get_namespace(&self) -> Option<&str> {
    None
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementData {
  Root(RootElement),
  Unstyled(UnstyledElement),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootElement;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnstyledElement;

#[derive(Debug, Serialize, Deserialize)]
pub struct CompiledDocument {
  pub root: Node<Element>,
  pub stylesheet: style::StyleSheet,

  #[serde(skip)]
  pub engine: rhai::Engine,
  #[serde(skip)]
  pub scope: RwLock<rhai::Scope<'static>>,
}

use std::io::prelude::*;

impl CompiledDocument {
  pub fn new(root: Node<Element>, stylesheet: style::StyleSheet) -> Self {
    Self {
      root,
      stylesheet,
      engine: rhai::Engine::default(),
      scope: RwLock::new(rhai::Scope::default()),
    }
  }

  #[must_use]
  pub fn save(&self) -> Vec<u8> {
    let mut buf = Vec::with_capacity(bincode::serialized_size(self).unwrap() as usize + MAGIC_BYTES.len());
    self.save_into(&mut buf);
    buf
  }

  pub fn save_into<W: Write>(&self, mut writer: W) {
    writer.write_all(MAGIC_BYTES).unwrap();
    bincode::serialize_into(writer, self).unwrap();
  }

  #[must_use]
  pub fn load(data: &[u8]) -> Self {
    Self::load_from(data)
  }

  #[must_use]
  pub fn load_from<R: Read>(mut reader: R) -> Self {
    let mut magic_bytes = [0; MAGIC_BYTES.len()];
    reader.read_exact(&mut magic_bytes).unwrap();

    if magic_bytes != MAGIC_BYTES {
      panic!("magic bytes don't match {:?} == {:?}", magic_bytes, MAGIC_BYTES);
    }

    let doc: CompiledDocument = bincode::deserialize_from(reader).unwrap();
    doc.init_yoga();
    doc
  }

  pub fn init_yoga(&self) {
    // Even though we don't need mutable access from the rust side,
    // we still want to make sure we are the only one with access to the
    // yoga nodes.
    for node in self.root.descendants().skip(1) {
      let inner = node.inner();
      let parent = inner.parent().unwrap().inner();
      unsafe {
        parent.yg.insert_child(*node.inner().yg, parent.yg.child_count());
      }
    }
  }

  pub fn compute_style(&self, width: f32, height: f32, direction: yoga::Direction) {
    for node in self.root.descendants() {
      node
        .inner_mut()
        .compute_attributes(&self.engine, &mut self.scope.write().unwrap());

      let mut computed = node.inner().computed;

      self.stylesheet.apply(&node, &mut computed);

      let mut el = node.inner_mut();
      el.computed = computed;
      el.prepare_yoga();
    }

    let mut root = self.root.inner_mut();
    unsafe {
      root.yg.calculate_layout(width, height, direction);
    }
  }

  pub fn query_selector(&self, selector: &str) -> Option<Node<Element>> {
    let mut input = cssparser::ParserInput::new(selector);
    let list = selectors::SelectorList::parse(
      &style::selectors::SelectorParser,
      &mut cssparser::Parser::new(&mut input),
    )
    .ok()?;

    let mut context = selectors::matching::MatchingContext::new(
      selectors::matching::MatchingMode::Normal,
      None,
      None,
      selectors::matching::QuirksMode::NoQuirks,
    );

    for node in self.root.descendants() {
      if selectors::matching::matches_selector_list(&list, &node, &mut context) {
        return Some(node);
      }
    }

    None
  }
}

impl Drop for CompiledDocument {
  fn drop(&mut self) {
    unsafe {
      self.root.inner_mut().yg.free_recursive();
    }
  }
}

impl selectors::Element for Node<Element> {
  type Impl = style::selectors::SelectorImpl;

  fn opaque(&self) -> ::selectors::OpaqueElement {
    selectors::OpaqueElement::new(self)
  }

  fn is_html_slot_element(&self) -> bool {
    false
  }

  fn parent_node_is_shadow_root(&self) -> bool {
    false
  }

  fn containing_shadow_host(&self) -> Option<Self> {
    None
  }

  fn parent_element(&self) -> Option<Self> {
    self.inner().parent().cloned()
  }

  fn prev_sibling_element(&self) -> Option<Self> {
    self.inner().previous_sibling().cloned()
  }

  fn next_sibling_element(&self) -> Option<Self> {
    self.inner().next_sibling().cloned()
  }

  fn is_empty(&self) -> bool {
    self.inner().first_child().is_none()
  }

  fn is_root(&self) -> bool {
    self.inner().parent().is_none()
  }

  fn is_html_element_in_html_document(&self) -> bool {
    false
  }

  fn has_local_name(&self, local_name: &str) -> bool {
    self.inner().get_local_name() == local_name
  }

  fn has_namespace(&self, ns: &str) -> bool {
    self.inner().get_namespace().map_or(false, |node_ns| node_ns == ns)
  }

  fn is_part(&self, _name: &String) -> bool {
    false
  }

  fn exported_part(&self, _name: &String) -> Option<String> {
    None
  }

  fn imported_part(&self, _name: &String) -> Option<String> {
    None
  }

  fn is_pseudo_element(&self) -> bool {
    false
  }

  fn is_same_type(&self, other: &Self) -> bool {
    let el = self.inner();
    let other = other.inner();

    el.get_local_name() == other.get_local_name() && el.get_namespace() == other.get_namespace()
  }

  fn is_link(&self) -> bool {
    false
  }

  fn has_id(&self, id: &String, case_sensitivity: selectors::attr::CaseSensitivity) -> bool {
    self
      .inner()
      .id
      .as_ref()
      .map_or(false, |id_attr| case_sensitivity.eq(id.as_bytes(), id_attr.as_bytes()))
  }

  fn has_class(&self, name: &String, case_sensitivity: selectors::attr::CaseSensitivity) -> bool {
    self
      .inner()
      .classes
      .iter()
      .any(|class| case_sensitivity.eq(class.as_bytes(), name.as_bytes()))
  }

  fn attr_matches(
    &self,
    _ns: &selectors::attr::NamespaceConstraint<&String>,
    _local_name: &String,
    _operation: &selectors::attr::AttrSelectorOperation<&String>,
  ) -> bool {
    false
  }

  fn match_pseudo_element(
    &self,
    _pe: &style::selectors::PseudoElement,
    _context: &mut selectors::matching::MatchingContext<Self::Impl>,
  ) -> bool {
    false
  }

  fn match_non_ts_pseudo_class<F>(
    &self,
    _pc: &style::selectors::PseudoClass,
    _context: &mut selectors::matching::MatchingContext<Self::Impl>,
    _flags_setter: &mut F,
  ) -> bool
  where
    F: FnMut(&Self, selectors::matching::ElementSelectorFlags),
  {
    false
  }
}

#[macro_export]
macro_rules! include_document {
  ($file:expr) => {
    ::std::sync::Arc::new(::project_a::dom::CompiledDocument::load(include_bytes!($file)))
  };
}
