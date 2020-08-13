use crate::{
  selectors::{SelectorImpl, SelectorParser},
  Declaration, StyleRule,
};

fn parse_yoga_value<'i, 't>(
  input: &mut cssparser::Parser<'i, 't>,
) -> Result<yoga::Value, cssparser::BasicParseError<'i>> {
  if input.try_parse(|input| input.expect_ident_matching("none")).is_ok() {
    Ok(yoga::Value::Undefined)
  } else if input.try_parse(|input| input.expect_ident_matching("auto")).is_ok() {
    Ok(yoga::Value::Auto)
  } else if let Ok(percent) = input.try_parse(|input| input.expect_percentage()) {
    Ok(yoga::Value::Percent(percent * 100.0))
  } else {
    let start_location = input.current_source_location();
    match input.next()? {
      cssparser::Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case("px") => Ok(yoga::Value::Px(*value)),

      token => Err(start_location.new_basic_unexpected_token_error(token.clone())),
    }
  }
}

impl Declaration {
  pub fn parse<'i, 't>(
    name: cssparser::CowRcStr<'i>,
    input: &mut cssparser::Parser<'i, 't>,
  ) -> Result<Self, cssparser::BasicParseError<'i>> {
    match &*name {
      "width" => Ok(Self::Width(parse_yoga_value(input)?)),
      "height" => Ok(Self::Height(parse_yoga_value(input)?)),
      "background-color" => {
        let start_location = input.current_source_location();
        let color = cssparser::Color::parse(input)?;
        match color {
          cssparser::Color::CurrentColor => {
            Err(start_location.new_basic_unexpected_token_error(cssparser::Token::Ident("currentcolor".into())))
          }

          cssparser::Color::RGBA(rgba) => Ok(Self::BackgroundColor(rgba.red, rgba.green, rgba.blue, rgba.alpha)),
        }
      }

      "margin-top" => Ok(Self::MarginTop(parse_yoga_value(input)?)),
      "margin-bottom" => Ok(Self::MarginBottom(parse_yoga_value(input)?)),
      "margin-left" => Ok(Self::MarginLeft(parse_yoga_value(input)?)),
      "margin-right" => Ok(Self::MarginRight(parse_yoga_value(input)?)),

      _ => Err(cssparser::BasicParseError {
        kind: cssparser::BasicParseErrorKind::QualifiedRuleInvalid,
        location: input.current_source_location(),
      }),
    }
  }
}

struct DeclarationParser;

impl<'i> cssparser::DeclarationParser<'i> for DeclarationParser {
  type Declaration = Declaration;
  type Error = selectors::parser::SelectorParseErrorKind<'i>;

  fn parse_value<'t>(
    &mut self,
    name: cssparser::CowRcStr<'i>,
    input: &mut cssparser::Parser<'i, 't>,
  ) -> Result<Self::Declaration, cssparser::ParseError<'i, Self::Error>> {
    Ok(Declaration::parse(name, input)?)
  }
}

impl<'i> cssparser::AtRuleParser<'i> for DeclarationParser {
  type PreludeNoBlock = ();
  type PreludeBlock = ();
  type AtRule = Declaration;
  type Error = selectors::parser::SelectorParseErrorKind<'i>;
}

pub struct QualifiedRuleParser;

impl<'i> cssparser::QualifiedRuleParser<'i> for QualifiedRuleParser {
  type Prelude = selectors::SelectorList<SelectorImpl>;
  type QualifiedRule = StyleRule;
  type Error = selectors::parser::SelectorParseErrorKind<'i>;

  fn parse_prelude<'t>(
    &mut self,
    input: &mut cssparser::Parser<'i, 't>,
  ) -> Result<Self::Prelude, cssparser::ParseError<'i, Self::Error>> {
    selectors::SelectorList::parse(&SelectorParser, input)
  }

  fn parse_block<'t>(
    &mut self,
    prelude: Self::Prelude,
    _location: cssparser::SourceLocation,
    input: &mut cssparser::Parser<'i, 't>,
  ) -> Result<Self::QualifiedRule, cssparser::ParseError<'i, Self::Error>> {
    let mut decl_parser = cssparser::DeclarationListParser::new(input, DeclarationParser);
    let mut declarations = Vec::new();
    while let Some(decl) = decl_parser.next() {
      let decl = decl.map_err(|(x, _)| x)?;
      declarations.push(decl);
    }

    Ok(StyleRule {
      selectors: prelude,
      properties: declarations,
    })
  }
}

impl<'i> cssparser::AtRuleParser<'i> for QualifiedRuleParser {
  type PreludeNoBlock = ();
  type PreludeBlock = ();
  type AtRule = StyleRule;
  type Error = selectors::parser::SelectorParseErrorKind<'i>;
}
