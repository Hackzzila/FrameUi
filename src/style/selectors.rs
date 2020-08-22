#[derive(Debug, Clone)]
pub struct SelectorParser;

impl<'i> selectors::Parser<'i> for SelectorParser {
  type Impl = SelectorImpl;
  type Error = selectors::parser::SelectorParseErrorKind<'i>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorImpl;

impl selectors::SelectorImpl for SelectorImpl {
  type AttrValue = String;
  type Identifier = String;
  type ClassName = String;
  type LocalName = String;
  type PartName = String;
  type NamespacePrefix = String;
  type NamespaceUrl = String;
  type BorrowedNamespaceUrl = str;
  type BorrowedLocalName = str;

  type NonTSPseudoClass = PseudoClass;
  type PseudoElement = PseudoElement;

  type ExtraMatchingData = ();
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum PseudoClass {}

impl selectors::parser::NonTSPseudoClass for PseudoClass {
  type Impl = SelectorImpl;

  fn is_active_or_hover(&self) -> bool {
    false
  }

  fn is_user_action_state(&self) -> bool {
    false
  }

  fn has_zero_specificity(&self) -> bool {
    false
  }
}

use std::fmt;

impl cssparser::ToCss for PseudoClass {
  fn to_css<W>(&self, _dest: &mut W) -> fmt::Result
  where
    W: fmt::Write,
  {
    match *self {}
  }
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum PseudoElement {}

impl cssparser::ToCss for PseudoElement {
  fn to_css<W>(&self, _dest: &mut W) -> fmt::Result
  where
    W: fmt::Write,
  {
    match *self {}
  }
}

impl selectors::parser::PseudoElement for PseudoElement {
  type Impl = SelectorImpl;
}
