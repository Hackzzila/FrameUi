use std::sync::RwLock;

use serde::{Serialize, Deserialize};
use indextree::{Arena, NodeId, Node};

pub const STRUCTURE_VERSION: u8 = 0;

fn safe_yoga_node_new() -> yoga::Node {
  unsafe { yoga::Node::new() }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Element {
  pub data: ElementData,
  pub classes: Vec<String>,
  pub id: Option<String>,
  pub style: Vec<style::StyleRule>,
  #[serde(skip, default = "safe_yoga_node_new")]
  pub yg: yoga::Node,
  #[serde(skip)]
  pub computed: style::ComputedStyle,
  #[serde(skip)]
  pub render: style::RenderStyle,
}

impl Element {
  pub fn new(data: ElementData) -> Self {
    Self {
      data,
      classes: Vec::new(),
      id: None,
      style: Vec::new(),
      yg: unsafe { yoga::Node::new() },
      computed: Default::default(),
      render: Default::default(),
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

  pub fn update_render(&mut self) {
    unsafe {
      self.render.width = self.yg.get_width();
      self.render.height = self.yg.get_height();
      self.render.top = self.yg.get_top();
      self.render.left = self.yg.get_left();
      self.render.background_color = self.computed.background_color;
    }
  }

  pub fn get_local_name(&self) -> &str {
    match self.data {
      ElementData::Root(..) => "#root",
      ElementData::Unstyled(..) => "Unstyled",
    }
  }

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
  pub version: u8,
  pub elements: RwLock<Arena<Element>>,
  pub root: NodeId,
  pub stylesheet: style::StyleSheet,
}

use std::io::prelude::*;

impl CompiledDocument {
  pub fn save(&self) -> Vec<u8> {
    bincode::serialize(self).unwrap()
  }

  pub fn save_into<W: Write>(&self, writer: W) {
    bincode::serialize_into(writer, self).unwrap();
  }

  pub fn load(data: &[u8]) -> Self {
    Self::load_from(data)
  }

  pub fn load_from<R: Read>(reader: R) -> Self {
    let doc: CompiledDocument = bincode::deserialize_from(reader).unwrap();
    doc.init_yoga();
    doc
  }

  pub fn init_yoga(&self) {
    // Even though we don't need mutable access from the rust side,
    // we still want to make sure we are the only one with access to the
    // yoga nodes.
    let arena = self.elements.write().unwrap();
    for node in self.root.descendants(&arena).skip(1) {
      let node = &arena[node];
      let parent = arena[node.parent().unwrap()].get();
      unsafe {
        parent.yg.insert_child(&node.get().yg, parent.yg.child_count());
      }
    }
  }

  pub fn compute_style(&self, width: f32, height: f32, direction: yoga::Direction) {
    let mut arena = self.elements.write().unwrap();
    let vec: Vec<_> = self.root.descendants(&arena).collect();
    for id in &vec {
      let node = &arena[*id];

      let mut computed = node.get().computed;

      let element = MatchingElement {
        elements: &*arena,
        node,
      };

      self.stylesheet.apply(&element, &mut computed);

      let node = arena[*id].get_mut();
      node.computed = computed;
      node.prepare_yoga();
    }

    let root = arena[self.root].get_mut();
    unsafe {
      root.yg.calculate_layout(width, height, direction);
    }

    for id in vec {
      arena[id].get_mut().update_render();
    }
  }

  pub fn query_selector(&self, selector: &str) -> Option<NodeId> {
    let mut input = cssparser::ParserInput::new(selector);
    let list = selectors::SelectorList::parse(&style::selectors::SelectorParser, &mut cssparser::Parser::new(&mut input)).ok()?;

    let mut context = selectors::matching::MatchingContext::new(
      selectors::matching::MatchingMode::Normal,
      None,
      None,
      selectors::matching::QuirksMode::NoQuirks,
    );

    let arena = self.elements.read().ok()?;
    for node in arena.iter() {
      if node.is_removed() {
        continue
      }

      let element = MatchingElement {
        elements: &*arena,
        node,
      };

      if selectors::matching::matches_selector_list(&list, &element, &mut context) {
        return arena.get_node_id(node);
      }
    }

    None
  }
}

impl Drop for CompiledDocument {
  fn drop(&mut self) {
    unsafe {
      self.elements.get_mut().unwrap()[self.root].get_mut().yg.free_recursive();
    }
  }
}

#[derive(Debug, Clone)]
pub struct MatchingElement<'a> {
  pub elements: &'a Arena<Element>,
  pub node: &'a Node<Element>,
}

impl<'a> selectors::Element for MatchingElement<'a> {
  type Impl = style::selectors::SelectorImpl;

  fn opaque(&self) -> ::selectors::OpaqueElement {
    selectors::OpaqueElement::new(self.node)
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
    self.node.parent().and_then(|parent| {
      Some(Self {
        elements: self.elements,
        node: &self.elements[parent],
      })
    })
  }

  fn prev_sibling_element(&self) -> Option<Self> {
    self.node.previous_sibling().and_then(|prev| {
      Some(Self {
        elements: self.elements,
        node: &self.elements[prev],
      })
    })
  }

  fn next_sibling_element(&self) -> Option<Self> {
    self.node.next_sibling().and_then(|next| {
      Some(Self {
        elements: self.elements,
        node: &self.elements[next],
      })
    })
  }

  fn is_empty(&self) -> bool {
    self.node.first_child().is_none()
  }

  fn is_root(&self) -> bool {
    match self.node.get().data {
      ElementData::Root(..) => true,
      _ => false,
    }
  }

  fn is_html_element_in_html_document(&self) -> bool {
    false
  }

  fn has_local_name(&self, local_name: &String) -> bool {
    self.node.get().get_local_name() == local_name
  }

  fn has_namespace(&self, ns: &String) -> bool {
    self.node.get().get_namespace().map_or(false, |node_ns| node_ns == ns)
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
    let el = self.node.get();
    let other = other.node.get();

    el.get_local_name() == other.get_local_name() && el.get_namespace() == other.get_namespace()
  }

  fn is_link(&self) -> bool {
    false
  }

  fn has_id(
    &self,
    id: &String,
    case_sensitivity: selectors::attr::CaseSensitivity,
  ) -> bool {
    self.node.get().id.as_ref().map_or(false, |id_attr| {
      case_sensitivity.eq(id.as_bytes(), id_attr.as_bytes())
    })
  }

  fn has_class(
    &self,
    name: &String,
    case_sensitivity: selectors::attr::CaseSensitivity,
  ) -> bool {
    self.node.get().classes.iter().any(|class| case_sensitivity.eq(class.as_bytes(), name.as_bytes()))
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
    F: FnMut(&Self, selectors::matching::ElementSelectorFlags) {
    false
  }
}

#[macro_export]
macro_rules! include_document {
  ($file:expr) => {
    ::std::sync::Arc::new(::project_a::dom::CompiledDocument::load(include_bytes!($file)))
  }
}
