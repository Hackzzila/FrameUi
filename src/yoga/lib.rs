pub mod sys {
  #![allow(non_upper_case_globals)]
  #![allow(non_camel_case_types)]
  #![allow(non_snake_case)]

  include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::fmt;
use std::ops::Deref;
use std::ffi::CStr;
use serde::{Serialize, Deserialize};

use sys::*;

macro_rules! yg_enum {
  ($name:ident) => {
    paste::item! {
      pub type $name = [<YG $name>];

      impl Into<&str> for $name {
        fn into(self) -> &'static str {
          unsafe {
            CStr::from_ptr(sys::[<YG $name ToString>](self)).to_str().unwrap()
          }
        }
      }

      impl fmt::Display for $name {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
          let string: &str = (*self).into();
          write!(f, "{}", string)
        }
      }
    }
  }
}

yg_enum!(Align);
yg_enum!(Dimension);
yg_enum!(Direction);
yg_enum!(Display);
yg_enum!(Edge);
yg_enum!(ExperimentalFeature);
yg_enum!(FlexDirection);
yg_enum!(Justify);
yg_enum!(LogLevel);
yg_enum!(MeasureMode);
yg_enum!(NodeType);
yg_enum!(Overflow);
yg_enum!(PositionType);
// yg_enum!(PrintOptions);
yg_enum!(Unit);
yg_enum!(Wrap);

bitflags::bitflags! {
  pub struct PrintOptions: u32 {
    const LAYOUT = 1;
    const STYLE = 2;
    const CHILDREN = 4;
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
  Px(f32),
  Percent(f32),
  Auto,
  Undefined,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Node {
  inner: YGNodeRef,
}

impl Node {
  pub unsafe fn new() -> Self {
    Self {
      inner: YGNodeNew(),
    }
  }

  pub unsafe fn free(&mut self) {
    YGNodeFree(**self)
  }

  pub unsafe fn free_recursive(&mut self) {
    YGNodeFreeRecursive(**self)
  }

  pub unsafe fn print(&self, options: PrintOptions) {
    YGNodePrint(**self, std::mem::transmute(options.bits()));
  }

  pub unsafe fn child_count(&self) -> u32 {
    YGNodeGetChildCount(**self)
  }

  pub unsafe fn get_child(&self, index: u32) -> Self {
    Self {
      inner: YGNodeGetChild(**self, index),
    }
  }

  // This should take &mut self, but that causes borrow issues when initializing dom...
  pub unsafe fn insert_child(&self, child: &YGNodeRef, index: u32) {
    YGNodeInsertChild(**self, *child, index);
  }

  pub unsafe fn set_width(&mut self, width: Value) {
    match width {
      Value::Px(v) => YGNodeStyleSetWidth(**self, v),
      Value::Percent(v) => YGNodeStyleSetWidthPercent(**self, v),
      Value::Auto => YGNodeStyleSetWidthAuto(**self),
      Value::Undefined => YGNodeStyleSetWidth(**self, f32::NAN),
    }
  }

  pub unsafe fn set_height(&mut self, height: Value) {
    match height {
      Value::Px(v) => YGNodeStyleSetHeight(**self, v),
      Value::Percent(v) => YGNodeStyleSetHeightPercent(**self, v),
      Value::Auto => YGNodeStyleSetHeightAuto(**self),
      Value::Undefined => YGNodeStyleSetHeight(**self, f32::NAN),
    }
  }

  pub unsafe fn set_margin(&mut self, edge: Edge, value: Value) {
    match value {
      Value::Px(v) => YGNodeStyleSetMargin(**self, edge, v),
      Value::Percent(v) => YGNodeStyleSetMarginPercent(**self, edge, v),
      Value::Auto => YGNodeStyleSetMarginAuto(**self, edge),
      Value::Undefined => YGNodeStyleSetMargin(**self, edge, f32::NAN),
    }
  }

  pub unsafe fn set_padding(&mut self, edge: Edge, value: Value) {
    match value {
      Value::Px(v) => YGNodeStyleSetPadding(**self, edge, v),
      Value::Percent(v) => YGNodeStyleSetPaddingPercent(**self, edge, v),
      Value::Auto => unimplemented!(),
      Value::Undefined => YGNodeStyleSetPadding(**self, edge, f32::NAN),
    }
  }

  pub unsafe fn set_position_type(&mut self, position: PositionType) {
    YGNodeStyleSetPositionType(**self, position);
  }

  pub unsafe fn set_display(&mut self, display: Display) {
    YGNodeStyleSetDisplay(**self, display);
  }

  pub unsafe fn set_justify_content(&mut self, justify_content: Justify) {
    YGNodeStyleSetJustifyContent(**self, justify_content);
  }

  pub unsafe fn calculate_layout(&mut self, available_width: f32, available_height: f32, owner_direction: Direction) {
    YGNodeCalculateLayout(**self, available_width, available_height, owner_direction);
  }

  pub unsafe fn get_top(&self) -> f32 {
    YGNodeLayoutGetTop(**self)
  }

  pub unsafe fn get_left(&self) -> f32 {
    YGNodeLayoutGetLeft(**self)
  }

  pub unsafe fn get_width(&self) -> f32 {
    YGNodeLayoutGetWidth(**self)
  }

  pub unsafe fn get_height(&self) -> f32 {
    YGNodeLayoutGetHeight(**self)
  }
}

unsafe impl Send for Node {}
unsafe impl Sync for Node {}

impl Deref for Node {
  type Target = YGNodeRef;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}
