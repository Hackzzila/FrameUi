use std::path::Path;

use codespan_reporting::{
  diagnostic::{Diagnostic, Label},
  files::{Files, SimpleFiles},
  term,
  term::termcolor::{ColorChoice, StandardStream},
};
use cssparser::ToCss;

use compiler::{compile, DiagnosticKind, Level};

struct DiagnosticPrinter {
  should_exit: bool,
  writer: StandardStream,
  config: codespan_reporting::term::Config,
  files: SimpleFiles<String, String>,
}

impl DiagnosticPrinter {
  fn new() -> Self {
    Self {
      should_exit: false,
      writer: StandardStream::stderr(ColorChoice::Auto),
      config: codespan_reporting::term::Config::default(),
      files: SimpleFiles::new(),
    }
  }
}

impl compiler::DiagnosticReporter for DiagnosticPrinter {
  type FileId = usize;

  fn add_file(&mut self, filename: String, source: String) -> Self::FileId {
    self.files.add(filename, source)
  }

  fn get_position(&mut self, file: &Self::FileId, line: usize, col: usize) -> usize {
    self.files.line_range(*file, line).unwrap().start + col - 1
  }

  fn get_line(&mut self, file: &Self::FileId, pos: usize) -> usize {
    self.files.line_index(*file, pos).unwrap()
  }

  fn add_diagnostic(&mut self, diagnostic: compiler::Diagnostic<Self::FileId>) {
    let location = diagnostic.location;

    let codespan_diagnostic = match diagnostic.min_level {
      Level::Bug => {
        self.should_exit = true;
        Diagnostic::bug()
      }

      Level::Error => {
        self.should_exit = true;
        Diagnostic::error()
      }

      Level::Warn => Diagnostic::warning(),
      Level::Info => Diagnostic::note(),
    };

    let diagnostic = match diagnostic.kind {
      DiagnosticKind::ExpectedSelfClosing { .. } => {
        let (file_id, pos) = location.unwrap();
        codespan_diagnostic
          .with_message("childless elements should be self-closing")
          .with_code("E0000")
          .with_labels(vec![
            Label::primary(file_id, pos - 1..pos).with_message("expected self-closing tag"),
            Label::secondary(file_id, pos - 1..pos).with_message("help: replace with `/>`"),
          ])
      }

      DiagnosticKind::ExpectedClosingTag { el } => {
        let (file_id, pos) = location.unwrap();
        codespan_diagnostic
          .with_message("element should have explicit closing tag")
          .with_code("E0000")
          .with_labels(vec![
            Label::primary(file_id, pos..pos).with_message("expected explicit closing tag"),
            Label::secondary(file_id, pos - 2..pos - 1).with_message("help: remove`/`"),
            Label::secondary(file_id, pos..pos).with_message(format!("help: add `</{}>`", el)),
          ])
      }

      DiagnosticKind::InvalidAttribute { el, attr } => {
        let (file_id, pos) = location.unwrap();
        codespan_diagnostic
          .with_message("invalid attribute")
          .with_code("E0000")
          .with_labels(vec![
            Label::primary(file_id, pos..pos).with_message(format!("invalid attribute `{}` for `{}`", attr, el))
          ])
      }

      DiagnosticKind::InvalidElement { el } => {
        let (file_id, pos) = location.unwrap();
        codespan_diagnostic
          .with_message("invalid element")
          .with_code("E0000")
          .with_labels(vec![
            Label::primary(file_id, pos..pos).with_message(format!("invalid element `{}`", el))
          ])
      }

      DiagnosticKind::InvalidContext { el, parent } => {
        let (file_id, pos) = location.unwrap();
        codespan_diagnostic
          .with_message("element found in invalid context")
          .with_code("E0000")
          .with_labels(vec![Label::primary(file_id, pos..pos)
            .with_message(format!("element `{}` is not allowed inside `{}`", el, parent))])
      }

      DiagnosticKind::CssParseError(err) => {
        let (file_id, pos) = location.unwrap();
        codespan_diagnostic
          .with_message("CSS parsing error")
          .with_code("E0000")
          .with_labels(vec![match err.0.kind {
            cssparser::ParseErrorKind::Basic(err) => match err {
              cssparser::BasicParseErrorKind::UnexpectedToken(token) => {
                let css = token.to_css_string();
                let end = pos + 1 + css.len();
                Label::primary(file_id, pos + 1..end).with_message(format!("unexpected token `{}`", css))
              }

              cssparser::BasicParseErrorKind::EndOfInput => {
                Label::primary(file_id, pos..pos).with_message("end of input".to_string())
              }

              cssparser::BasicParseErrorKind::AtRuleInvalid(rule) => {
                let beg = pos - rule.len() - 1;
                Label::primary(file_id, beg..pos).with_message(format!("at-rule `{}` invalid", rule))
              }

              cssparser::BasicParseErrorKind::AtRuleBodyInvalid => {
                Label::primary(file_id, pos..pos).with_message("at-rule body invalid".to_string())
              }

              cssparser::BasicParseErrorKind::QualifiedRuleInvalid => {
                Label::primary(file_id, pos..pos).with_message("qualified rule invalid".to_string())
              }
            },

            cssparser::ParseErrorKind::Custom(..) => unimplemented!(),
          }])
      }

      DiagnosticKind::SassParseError(err) => {
        let (file_id, pos) = location.unwrap();
        codespan_diagnostic
          .with_message("libsass error")
          .with_code("E0000")
          .with_labels(vec![Label::primary(file_id, pos..pos).with_message(err)])
      }

      kind => {
        if let Some((file_id, pos)) = location {
          codespan_diagnostic.with_labels(vec![Label::primary(file_id, pos..pos).with_message(kind.to_string())])
        } else {
          codespan_diagnostic.with_message(kind.to_string())
        }
      }
    };

    term::emit(&mut self.writer.lock(), &self.config, &self.files, &diagnostic).unwrap();
  }

  fn checkpoint(&mut self) -> Result<(), ()> {
    if self.should_exit {
      Err(())
    } else {
      Ok(())
    }
  }
}

use clap::{App, Arg};

fn main() {
  let matches = App::new(env!("CARGO_PKG_NAME"))
    .version(env!("CARGO_PKG_VERSION"))
    .author(env!("CARGO_PKG_AUTHORS"))
    .about(env!("CARGO_PKG_DESCRIPTION"))
    .arg(
      Arg::with_name("INPUT")
        .help("Sets the input file to use")
        .required(true)
        .index(1),
    )
    .arg(
      Arg::with_name("output")
        .short("o")
        .long("output")
        .value_name("FILE")
        .help("Sets the output file")
        .required(true)
        .takes_value(true),
    )
    .get_matches();

  let mut printer = DiagnosticPrinter::new();
  let result = compile(&Path::new(matches.value_of("INPUT").unwrap()), &mut printer);
  if let Ok(doc) = result {
    let f = std::fs::File::create(matches.value_of("output").unwrap()).unwrap();
    doc.save_into(f);
  }
}
