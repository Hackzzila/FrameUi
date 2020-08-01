use compiler::compile;

use compiler::{Error, DiagnosticData};
use std::io::prelude::*;
use url::Url;
use std::path::Path;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term;

struct DiagnosticPrinter {
  writer: StandardStream,
  config: codespan_reporting::term::Config,
  files: SimpleFiles<String, String>,
}

impl DiagnosticPrinter {
  fn new() -> Self {
    Self {
      writer: StandardStream::stderr(ColorChoice::Always),
      config: codespan_reporting::term::Config::default(),
      files: SimpleFiles::new(),
    }
  }

  fn handle_error(&self, err: Error<usize>) {
    let diagnostic = match err {
      Error::ParseError(err, file_id, pos) => {
        Diagnostic::error()
          .with_message("parsing error")
          .with_code("E0000")
          .with_labels(vec![
            Label::primary(file_id, pos..pos).with_message(err.to_string()),
          ])
      }

      _ => {
        Diagnostic::error()
          .with_message(err.to_string())
          .with_code("E0000")
      }
    };

    term::emit(&mut self.writer.lock(), &self.config, &self.files, &diagnostic).unwrap();
  }
}

impl compiler::DiagnosticReporter for DiagnosticPrinter {
  type FileId = usize;

  fn add_file(&mut self, url: &Url) -> Result<Self::FileId, Error<Self::FileId>> {
    let file_name = url.path_segments().unwrap().next_back().unwrap().to_string();

    let mut reader = compiler::Reader::get(url)?;
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;

    Ok(self.files.add(file_name, contents))
  }

  fn add_diagnostic(&mut self, diagnostic: compiler::Diagnostic<Self::FileId>) -> Result<(), Error<Self::FileId>> {
    let file_id = diagnostic.file_id;
    let pos = diagnostic.pos;

    let diagnostic = match diagnostic.data {
      // Diag::ParseError(err) => {
      //   Diagnostic::error()
      //     .with_message("parsing error")
      //     .with_code("E0000")
      //     .with_labels(vec![
      //       Label::primary(file_id, pos..pos).with_message(err.to_string()),
      //     ])
      // }

      DiagnosticData::ExpectedSelfClosing => {
        Diagnostic::warning()
          .with_message("childless elements should be self-closing")
          .with_code("E0000")
          .with_labels(vec![
            Label::primary(file_id, pos-1..pos).with_message("expected self-closing tag"),
            Label::secondary(file_id, pos-1..pos).with_message("help: replace with `/>`"),
          ])
      }

      DiagnosticData::ExpectedClosingTag(name) => {
        Diagnostic::warning()
          .with_message("element should have explicit closing tag")
          .with_code("E0000")
          .with_labels(vec![
            Label::primary(file_id, pos..pos).with_message("expected explicit closing tag"),
            Label::secondary(file_id, pos-2..pos-1).with_message("help: remove`/`"),
            Label::secondary(file_id, pos..pos).with_message(format!("help: add `</{}>`", name)),
          ])
      }

      DiagnosticData::InvalidAttribute(attr, el) => {
        Diagnostic::error()
          .with_message("invalid attribute")
          .with_code("E0000")
          .with_labels(vec![
            Label::primary(file_id, pos..pos).with_message(format!("invalid attribute `{}` for `{}`", attr, el)),
          ])
      }

      DiagnosticData::InvalidElement(name) => {
        Diagnostic::error()
          .with_message("invalid element")
          .with_code("E0000")
          .with_labels(vec![
            Label::primary(file_id, pos..pos).with_message(format!("invalid element `{}`", name)),
          ])
      }

      DiagnosticData::InvalidContext(name, ty, parent) => {
        Diagnostic::error()
          .with_message("element found in invalid context")
          .with_code("E0000")
          .with_labels(vec![
            Label::primary(file_id, pos..pos).with_message(format!("element `{}` of type `{}` is not allowed inside `{}`", name, ty, parent)),
          ])
      }
    };

    term::emit(&mut self.writer.lock(), &self.config, &self.files, &diagnostic).unwrap();

    Ok(())
  }
}

use clap::{Arg, App};

fn main() {
  let matches = App::new("My Super Program")
    .version(env!("CARGO_PKG_VERSION"))
    .author(env!("CARGO_PKG_AUTHORS"))
    .about(env!("CARGO_PKG_DESCRIPTION"))
    .arg(Arg::with_name("INPUT")
      .help("Sets the input file to use")
      .required(true)
      .index(1))
    .arg(Arg::with_name("output")
      .short("o")
      .long("output")
      .value_name("FILE")
      .help("Sets the output file")
      .required(true)
      .takes_value(true))
    .get_matches();

  let mut printer = DiagnosticPrinter::new();
  let v = compile(&Path::new(matches.value_of("INPUT").unwrap()), &mut printer);
  match v {
    Ok(doc) => {
      let f = std::fs::File::create(matches.value_of("output").unwrap()).unwrap();
      doc.save_into(f);
    },

    Err(e) => {
      printer.handle_error(e);
    }
  }
}
