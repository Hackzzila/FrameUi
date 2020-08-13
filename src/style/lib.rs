use serde::{Deserialize, Serialize};

pub mod parser;
pub mod selectors;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RenderStyle {
  pub width: f32,
  pub height: f32,
  pub top: f32,
  pub left: f32,
  pub background_color: (u8, u8, u8, u8),
}

impl Default for RenderStyle {
  fn default() -> Self {
    Self {
      width: f32::NAN,
      height: f32::NAN,
      top: f32::NAN,
      left: f32::NAN,
      background_color: (0, 0, 0, 0),
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ComputedStyle {
  pub width: yoga::Value,
  pub height: yoga::Value,
  pub background_color: (u8, u8, u8, u8),
  pub margin_top: yoga::Value,
  pub margin_bottom: yoga::Value,
  pub margin_left: yoga::Value,
  pub margin_right: yoga::Value,
}

impl Default for ComputedStyle {
  fn default() -> Self {
    Self {
      width: yoga::Value::Auto,
      height: yoga::Value::Auto,
      background_color: (0, 0, 0, 0),
      margin_top: yoga::Value::Px(0.0),
      margin_bottom: yoga::Value::Px(0.0),
      margin_left: yoga::Value::Px(0.0),
      margin_right: yoga::Value::Px(0.0),
    }
  }
}

pub type ParserInput<'i> = cssparser::ParserInput<'i>;
pub type Error<'i> = (
  cssparser::ParseError<'i, ::selectors::parser::SelectorParseErrorKind<'i>>,
  &'i str,
);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleSheet {
  pub rules: Vec<StyleRule>,
}

impl StyleSheet {
  pub fn new() -> Self {
    Self { rules: Vec::new() }
  }

  pub fn create_parser_input(input: &str) -> ParserInput<'_> {
    cssparser::ParserInput::new(input)
  }

  pub fn create_parser_input_with_line_offset(input: &str, offset: u32) -> ParserInput<'_> {
    cssparser::ParserInput::new_with_line_number_offset(input, offset)
  }

  pub fn parse<'i>(&mut self, input: &mut cssparser::ParserInput<'i>) -> Result<(), Error<'i>> {
    let mut parser = cssparser::Parser::new(input);

    let rule_list_parser = cssparser::RuleListParser::new_for_stylesheet(&mut parser, parser::QualifiedRuleParser);
    for rule in rule_list_parser {
      self.rules.push(rule?);
    }

    Ok(())
  }

  pub fn apply<E: ::selectors::Element<Impl = selectors::SelectorImpl>>(
    &self,
    element: &E,
    computed: &mut ComputedStyle,
  ) {
    self.rules.iter().for_each(|x| x.apply(element, computed));
  }
}

impl Default for StyleSheet {
  fn default() -> StyleSheet {
    StyleSheet::new()
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "SerdeStyleRule", into = "SerdeStyleRule")]
pub struct StyleRule {
  pub selectors: ::selectors::SelectorList<selectors::SelectorImpl>,
  pub properties: Vec<Declaration>,
}

impl StyleRule {
  pub fn apply<E: ::selectors::Element<Impl = selectors::SelectorImpl>>(
    &self,
    element: &E,
    computed: &mut ComputedStyle,
  ) {
    let mut context = ::selectors::matching::MatchingContext::new(
      ::selectors::matching::MatchingMode::Normal,
      None,
      None,
      ::selectors::matching::QuirksMode::NoQuirks,
    );

    if ::selectors::matching::matches_selector_list(&self.selectors, element, &mut context) {
      self.properties.iter().for_each(|x| x.apply(computed));
    }
  }
}

use cssparser::ToCss;

#[derive(Serialize, Deserialize)]
struct SerdeStyleRule {
  selectors: String,
  properties: Vec<Declaration>,
}

impl From<StyleRule> for SerdeStyleRule {
  fn from(rule: StyleRule) -> Self {
    Self {
      selectors: rule.selectors.to_css_string(),
      properties: rule.properties,
    }
  }
}

impl From<SerdeStyleRule> for StyleRule {
  fn from(rule: SerdeStyleRule) -> Self {
    let mut input = cssparser::ParserInput::new(&rule.selectors);
    let selectors =
      ::selectors::SelectorList::parse(&selectors::SelectorParser, &mut cssparser::Parser::new(&mut input)).unwrap();
    StyleRule {
      selectors,
      properties: rule.properties,
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Declaration {
  Width(yoga::Value),
  Height(yoga::Value),
  BackgroundColor(u8, u8, u8, u8),
  MarginTop(yoga::Value),
  MarginBottom(yoga::Value),
  MarginLeft(yoga::Value),
  MarginRight(yoga::Value),
}

impl Declaration {
  pub fn apply(&self, computed: &mut ComputedStyle) {
    match self {
      Self::Width(value) => computed.width = *value,
      Self::Height(value) => computed.height = *value,
      Self::BackgroundColor(r, g, b, a) => computed.background_color = (*r, *g, *b, *a),
      Self::MarginTop(value) => computed.margin_top = *value,
      Self::MarginBottom(value) => computed.margin_bottom = *value,
      Self::MarginLeft(value) => computed.margin_left = *value,
      Self::MarginRight(value) => computed.margin_right = *value,
    }
  }
}
