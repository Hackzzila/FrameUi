use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug)]
pub struct Node<T>(Arc<RwLock<NodeInner<T>>>);

impl<T> Clone for Node<T> {
  fn clone(&self) -> Self {
    Node(Arc::clone(&self.0))
  }
}

impl<T> PartialEq for Node<T> {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.0, &other.0)
  }
}
impl<T> Eq for Node<T> {}

#[derive(Debug)]
pub struct NodeInner<T> {
  parent: Option<Node<T>>,
  previous_sibling: Option<Node<T>>,
  next_sibling: Option<Node<T>>,
  first_child: Option<Node<T>>,
  last_child: Option<Node<T>>,

  pub data: T,
}

impl<T> NodeInner<T> {
  pub fn parent(&self) -> Option<&Node<T>> {
    self.parent.as_ref()
  }

  pub fn previous_sibling(&self) -> Option<&Node<T>> {
    self.previous_sibling.as_ref()
  }

  pub fn next_sibling(&self) -> Option<&Node<T>> {
    self.next_sibling.as_ref()
  }

  pub fn first_child(&self) -> Option<&Node<T>> {
    self.first_child.as_ref()
  }

  pub fn last_child(&self) -> Option<&Node<T>> {
    self.last_child.as_ref()
  }
}

impl<T> std::ops::Deref for NodeInner<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.data
  }
}

impl<T> std::ops::DerefMut for NodeInner<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.data
  }
}

impl<T> Node<T> {
  pub fn new(data: T) -> Self {
    Self(Arc::new(RwLock::new(NodeInner {
      parent: None,
      previous_sibling: None,
      next_sibling: None,
      first_child: None,
      last_child: None,
      data,
    })))
  }

  pub fn inner(&self) -> RwLockReadGuard<'_, NodeInner<T>> {
    self.0.read().unwrap()
  }

  pub fn inner_mut(&self) -> RwLockWriteGuard<'_, NodeInner<T>> {
    self.0.write().unwrap()
  }

  pub fn append(&self, data: T) -> Node<T> {
    let mut inner = self.0.write().unwrap();
    match inner.last_child.as_ref() {
      Some(last) => {
        let node = Node(Arc::new(RwLock::new(NodeInner {
          parent: Some(self.clone()),
          previous_sibling: Some(last.clone()),
          next_sibling: None,
          first_child: None,
          last_child: None,
          data,
        })));

        last.0.write().unwrap().next_sibling = Some(node.clone());
        inner.last_child = Some(node.clone());

        node
      }

      None => {
        let node = Node(Arc::new(RwLock::new(NodeInner {
          parent: Some(self.clone()),
          previous_sibling: None,
          next_sibling: None,
          first_child: None,
          last_child: None,
          data,
        })));

        inner.first_child = Some(node.clone());
        inner.last_child = Some(node.clone());

        node
      }
    }
  }

  pub fn children(&self) -> Children<T> {
    Children {
      current: self.inner().first_child().cloned(),
    }
  }

  pub fn traverse(&self) -> Traverse<T> {
    Traverse::new(self.clone())
  }

  pub fn descendants(&self) -> Descendants<T> {
    Descendants::new(self.clone())
  }
}

pub struct Children<T> {
  current: Option<Node<T>>,
}

impl<T> std::iter::Iterator for Children<T> {
  type Item = Node<T>;

  fn next(&mut self) -> Option<Node<T>> {
    let current = self.current.clone();
    self.current = current
      .as_ref()
      .and_then(|x| x.inner().next_sibling().cloned());
    current
  }
}

#[derive(Clone)]
/// An iterator of the IDs of a given node and its descendants, as a pre-order depth-first search where children are visited in insertion order.
///
/// i.e. node -> first child -> second child
pub struct Descendants<T>(Traverse<T>);

impl<'a, T> Descendants<T> {
  pub(crate) fn new(current: Node<T>) -> Self {
    Self(Traverse::new(current))
  }
}

impl<'a, T> Iterator for Descendants<T> {
  type Item = Node<T>;

  fn next(&mut self) -> Option<Node<T>> {
    self.0.find_map(|edge| match edge {
      NodeEdge::Start(node) => Some(node),
      NodeEdge::End(_) => None,
    })
  }
}

#[derive(Debug)]
/// Indicator if the node is at a start or endpoint of the tree
pub enum NodeEdge<T> {
  /// Indicates that start of a node that has children.
  ///
  /// Yielded by `Traverse::next()` before the node’s descendants. In HTML or
  /// XML, this corresponds to an opening tag like `<div>`.
  Start(Node<T>),

  /// Indicates that end of a node that has children.
  ///
  /// Yielded by `Traverse::next()` after the node’s descendants. In HTML or
  /// XML, this corresponds to a closing tag like `</div>`
  End(Node<T>),
}

impl<T> Clone for NodeEdge<T> {
  fn clone(&self) -> Self {
    match self {
      Self::Start(n) => Self::Start(n.clone()),
      Self::End(n) => Self::End(n.clone()),
    }
  }
}

#[derive(Clone)]
/// An iterator of the "sides" of a node visited during a depth-first pre-order traversal,
/// where node sides are visited start to end and children are visited in insertion order.
///
/// i.e. node.start -> first child -> second child -> node.end
pub struct Traverse<T> {
  root: Node<T>,
  next: Option<NodeEdge<T>>,
}

impl<T> Traverse<T> {
  pub(crate) fn new(current: Node<T>) -> Self {
    Self {
      root: current.clone(),
      next: Some(NodeEdge::Start(current)),
    }
  }

  /// Calculates the next node.
  fn next_of_next(&self, next: NodeEdge<T>) -> Option<NodeEdge<T>> {
    match next {
      NodeEdge::Start(node) => match node.inner().first_child() {
        Some(first_child) => Some(NodeEdge::Start(first_child.clone())),
        None => Some(NodeEdge::End(node.clone())),
      },
      NodeEdge::End(node) => {
        if node == self.root {
          return None;
        }
        match node.inner().next_sibling.clone() {
          Some(next_sibling) => Some(NodeEdge::Start(next_sibling)),
          // `node.parent()` here can only be `None` if the tree has
          // been modified during iteration, but silently stoping
          // iteration seems a more sensible behavior than panicking.
          None => node.inner().parent.clone().map(NodeEdge::End),
        }
      }
    }
  }
}

impl<T> Iterator for Traverse<T> {
  type Item = NodeEdge<T>;

  fn next(&mut self) -> Option<NodeEdge<T>> {
    let next = self.next.take()?;
    self.next = self.next_of_next(next.clone());
    Some(next)
  }
}

mod serde {
  use super::{Node, NodeInner};

  use std::{
    fmt,
    marker::PhantomData,
    sync::{Arc, RwLock},
  };

  use serde::{
    de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, SerializeStruct, Serializer},
  };

  impl<T: Serialize> Serialize for Node<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      let mut state = serializer.serialize_struct("Node", 2)?;
      state.serialize_field("data", &self.inner().data)?;
      state.serialize_field("children", &NodeSerializeChildren(self))?;
      state.end()
    }
  }

  struct NodeSerializeChildren<'a, T>(&'a Node<T>);

  impl<T: Serialize> Serialize for NodeSerializeChildren<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
    {
      let mut seq = serializer.serialize_seq(Some(self.0.children().count()))?;
      for node in self.0.children() {
        seq.serialize_element(&node)?;
      }
      seq.end()
    }
  }

  struct PartialParent<T> {
    first_child: Option<Node<T>>,
    last_child: Option<Node<T>>,
  }

  impl<'de, T: Deserialize<'de>> Deserialize<'de> for PartialParent<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      struct ChildVisitor<T> {
        last_node: Option<Node<T>>,
      }

      impl<'de, T: Deserialize<'de>> Visitor<'de> for ChildVisitor<T> {
        type Value = PartialParent<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
          formatter.write_str("struct Duration")
        }

        fn visit_seq<V>(mut self, mut seq: V) -> Result<PartialParent<T>, V::Error>
        where
          V: SeqAccess<'de>,
        {
          let mut first_child = None;

          while let Some(node) = seq.next_element::<Node<T>>()? {
            if first_child.is_none() {
              first_child = Some(node.clone());
            }

            if let Some(last_node) = self.last_node {
              last_node.inner_mut().next_sibling = Some(node.clone());
              node.inner_mut().previous_sibling = Some(last_node.clone());
            }

            self.last_node = Some(node);
          }

          Ok(PartialParent {
            first_child,
            last_child: self.last_node,
          })
        }
      }

      deserializer.deserialize_seq(ChildVisitor { last_node: None })
    }
  }

  impl<'de, T: Deserialize<'de>> Deserialize<'de> for Node<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
      D: Deserializer<'de>,
    {
      #[derive(serde::Deserialize)]
      #[serde(field_identifier, rename_all = "lowercase")]
      enum Field {
        Data,
        Children,
      }

      struct NodeVisitor<T> {
        _phantom: PhantomData<T>,
      }

      impl<T> NodeVisitor<T> {
        fn construct(data: T, partial: PartialParent<T>) -> Node<T> {
          let parent = Node(Arc::new(RwLock::new(NodeInner {
            parent: None,
            previous_sibling: None,
            next_sibling: None,
            first_child: partial.first_child,
            last_child: partial.last_child,

            data,
          })));

          for child in parent.children() {
            child.inner_mut().parent = Some(parent.clone());
          }

          parent
        }
      }

      impl<'de, T: Deserialize<'de>> Visitor<'de> for NodeVisitor<T> {
        type Value = Node<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
          formatter.write_str("struct Node")
        }

        fn visit_seq<V>(self, mut seq: V) -> Result<Node<T>, V::Error>
        where
          V: SeqAccess<'de>,
        {
          let data = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
          let partial = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
          Ok(Self::construct(data, partial))
        }

        fn visit_map<V>(self, mut map: V) -> Result<Node<T>, V::Error>
        where
          V: MapAccess<'de>,
        {
          let mut data = None;
          let mut partial = None;
          while let Some(key) = map.next_key()? {
            match key {
              Field::Data => {
                if data.is_some() {
                  return Err(de::Error::duplicate_field("data"));
                }
                data = Some(map.next_value()?);
              }

              Field::Children => {
                if partial.is_some() {
                  return Err(de::Error::duplicate_field("children"));
                }
                partial = Some(map.next_value()?);
              }
            }
          }

          let data = data.ok_or_else(|| de::Error::missing_field("data"))?;
          let partial = partial.ok_or_else(|| de::Error::missing_field("children"))?;

          Ok(Self::construct(data, partial))
        }
      }

      const FIELDS: &[&str] = &["data", "children"];
      deserializer.deserialize_struct("NodeInner", FIELDS, NodeVisitor::<T> { _phantom: PhantomData })
    }
  }
}
