use std::{
  fmt,
  fs::File,
  io,
  io::{prelude::*, BufReader},
  path::Path,
  sync::RwLock,
};

use indextree::{Arena, NodeId};
use quick_xml::events::{BytesStart, Event};
use reqwest::blocking::{get, Response};
use url::Url;

use dom::{CompiledDocument, Element, ElementData, RootElement, UnstyledElement, STRUCTURE_VERSION};
use style::StyleSheet;

#[path = "style.rs"]
mod _style;

pub trait IntoUrl {
  fn into_url(&self) -> Result<Url, DiagnosticKind>;
}

impl IntoUrl for &str {
  fn into_url(&self) -> Result<Url, DiagnosticKind> {
    Ok(Url::parse(self)?)
  }
}

impl<T: AsRef<Path>> IntoUrl for &T {
  fn into_url(&self) -> Result<Url, DiagnosticKind> {
    let path = self.as_ref().canonicalize()?;
    Ok(Url::from_file_path(path).unwrap())
  }
}

#[derive(Debug, Clone)]
pub enum Level {
  Bug,
  Error,
  Warn,
  Info,
}

#[derive(Debug)]
pub struct Diagnostic<'i, FileId: fmt::Debug> {
  pub kind: DiagnosticKind<'i>,
  pub location: Option<(FileId, usize)>,
  pub min_level: Level,
}

impl<FileId: fmt::Debug> fmt::Display for Diagnostic<'_, FileId> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.kind.fmt(f)
  }
}

#[derive(Debug)]
pub enum DiagnosticKind<'i> {
  InvalidElement { el: String },
  InvalidContext { el: String, parent: String },
  InvalidAttribute { el: String, attr: String },
  ExpectedSelfClosing { el: String },
  ExpectedClosingTag { el: String },

  UnexpectedText,
  UnexpectedCData,
  UnexpectedDecl,
  UnexpectedPI,
  UnexpectedDocType,
  UnexpectedEof,

  IOError(io::Error),
  ReqwestError(reqwest::Error),
  ParseError(quick_xml::Error),
  UrlParseError(url::ParseError),
  CssParseError(style::Error<'i>),
  SassParseError(String),
  MissingNode(NodeId, &'static str, u32, u32),
}

impl fmt::Display for DiagnosticKind<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InvalidElement { el } => write!(f, "invalid element {}", el),
      Self::InvalidContext { el, parent } => write!(f, "element `{}` is not allowed inside `{}`", el, parent),
      Self::InvalidAttribute { el, attr } => write!(f, "invalid attribute `{}` for `{}`", attr, el),
      Self::ExpectedSelfClosing { el } => write!(f, "childless element `{}` should be self-closing", el),
      Self::ExpectedClosingTag { el } => write!(f, "element `{}` should have explicit closing tag", el),

      Self::UnexpectedText => write!(f, "unexpected text"),
      Self::UnexpectedCData => write!(f, "unexpected CDATA"),
      Self::UnexpectedDecl => write!(f, "unexpected declaration"),
      Self::UnexpectedPI => write!(f, "unexpected processing instruction"),
      Self::UnexpectedDocType => write!(f, "unexpected DOCTYPE"),
      Self::UnexpectedEof => write!(f, "unexpected EOF"),

      Self::IOError(e) => e.fmt(f),
      Self::ReqwestError(e) => e.fmt(f),
      Self::ParseError(e) => e.fmt(f),
      Self::UrlParseError(e) => e.fmt(f),
      Self::CssParseError(e) => write!(f, "{:?}", e),
      Self::SassParseError(e) => e.fmt(f),
      Self::MissingNode(node, file, line, col) => write!(f, "missing node `{}`, {}:{}:{} ", node, file, line, col),
    }
  }
}

impl<'i> From<io::Error> for DiagnosticKind<'i> {
  fn from(e: io::Error) -> DiagnosticKind<'i> {
    DiagnosticKind::IOError(e)
  }
}

impl<'i> From<reqwest::Error> for DiagnosticKind<'i> {
  fn from(e: reqwest::Error) -> DiagnosticKind<'i> {
    DiagnosticKind::ReqwestError(e)
  }
}

impl<'i> From<url::ParseError> for DiagnosticKind<'i> {
  fn from(e: url::ParseError) -> DiagnosticKind<'i> {
    DiagnosticKind::UrlParseError(e)
  }
}

impl<'i> From<quick_xml::Error> for DiagnosticKind<'i> {
  fn from(e: quick_xml::Error) -> DiagnosticKind<'i> {
    DiagnosticKind::ParseError(e)
  }
}

enum Reader {
  File(BufReader<File>),
  Network(BufReader<Response>),
}

impl Reader {
  pub fn get(url: &Url) -> Result<Reader, DiagnosticKind> {
    if url.scheme() == "file" {
      let file = File::open(url.to_file_path().unwrap())?;
      let buf = BufReader::new(file);
      Ok(Reader::File(buf))
    } else {
      let resp = get(url.clone())?;
      let buf = BufReader::new(resp);
      Ok(Reader::Network(buf))
    }
  }
}

impl Read for Reader {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    match self {
      Reader::File(buf_reader) => buf_reader.read(buf),
      Reader::Network(buf_reader) => buf_reader.read(buf),
    }
  }
}

impl BufRead for Reader {
  fn consume(&mut self, amt: usize) {
    match self {
      Reader::File(buf) => buf.consume(amt),
      Reader::Network(buf) => buf.consume(amt),
    }
  }

  fn fill_buf(&mut self) -> io::Result<&[u8]> {
    match self {
      Reader::File(buf) => buf.fill_buf(),
      Reader::Network(buf) => buf.fill_buf(),
    }
  }
}

pub trait DiagnosticReporter {
  type FileId: fmt::Debug + Clone;
  fn add_file(&mut self, filename: String, source: String) -> Self::FileId;
  fn add_diagnostic(&mut self, diagnostic: Diagnostic<Self::FileId>);
  fn get_position(&mut self, file: &Self::FileId, line: usize, col: usize) -> usize;
  fn get_line(&mut self, file: &Self::FileId, pos: usize) -> usize;
  fn checkpoint(&mut self) -> Result<(), ()>;
}

struct Context<'r, FileId: fmt::Debug + Clone> {
  body: Arena<dom::Element>,
  root: NodeId,
  reporter: &'r mut dyn DiagnosticReporter<FileId = FileId>,
  stylesheet: StyleSheet,
}

#[macro_export]
macro_rules! handle_error_with_location {
  ($ctx:ident, $file_id:ident, $reader:ident) => {
    |e| {
      $ctx.reporter.add_diagnostic(crate::Diagnostic {
        location: Some(($file_id.clone(), $reader.buffer_position())),
        min_level: crate::Level::Error,
        kind: e.into(),
      })
    }
  };
}

#[macro_export]

macro_rules! handle_error {
  ($reporter:ident) => {
    |e| {
      $reporter.add_diagnostic(crate::Diagnostic {
        location: None,
        min_level: crate::Level::Error,
        kind: e.into(),
      })
    }
  };
}

impl<'r, FileId: fmt::Debug + Clone> Context<'r, FileId> {
  fn handle_event<R: BufRead>(
    &mut self,
    event: Event,
    file_id: &FileId,
    reader: &mut quick_xml::Reader<R>,
  ) -> Result<(), ()> {
    match event {
      Event::Text(text) => {
        let text = text
          .unescaped()
          .map_err(handle_error_with_location!(self, file_id, reader))?;
        let text = reader
          .decode(&text)
          .map_err(handle_error_with_location!(self, file_id, reader))?;

        if text.trim().is_empty() {
          Ok(())
        } else {
          self.reporter.add_diagnostic(Diagnostic {
            min_level: Level::Error,
            location: Some((file_id.clone(), reader.buffer_position())),
            kind: DiagnosticKind::UnexpectedText,
          });
          Err(())
        }
      }

      Event::CData(..) => {
        self.reporter.add_diagnostic(Diagnostic {
          min_level: Level::Error,
          location: Some((file_id.clone(), reader.buffer_position())),
          kind: DiagnosticKind::UnexpectedCData,
        });
        Err(())
      }

      Event::Decl(..) => {
        self.reporter.add_diagnostic(Diagnostic {
          min_level: Level::Error,
          location: Some((file_id.clone(), reader.buffer_position())),
          kind: DiagnosticKind::UnexpectedDecl,
        });
        Err(())
      }

      Event::PI(..) => {
        self.reporter.add_diagnostic(Diagnostic {
          min_level: Level::Error,
          location: Some((file_id.clone(), reader.buffer_position())),
          kind: DiagnosticKind::UnexpectedPI,
        });
        Err(())
      }

      Event::DocType(..) => {
        self.reporter.add_diagnostic(Diagnostic {
          min_level: Level::Error,
          location: Some((file_id.clone(), reader.buffer_position())),
          kind: DiagnosticKind::UnexpectedDocType,
        });
        Err(())
      }

      Event::Eof => {
        self.reporter.add_diagnostic(Diagnostic {
          min_level: Level::Error,
          location: Some((file_id.clone(), reader.buffer_position())),
          kind: DiagnosticKind::UnexpectedEof,
        });
        Err(())
      }

      Event::Comment(..) => Ok(()),

      _ => unimplemented!(),
    }
  }

  fn compile_root<R: BufRead>(
    &mut self,
    reader: &mut quick_xml::Reader<R>,
    buf: &mut Vec<u8>,
    url: &Url,
    file_id: &FileId,
  ) -> Result<(), ()> {
    buf.clear();

    let mut found_frame = false;
    loop {
      match reader
        .read_event(buf)
        .map_err(handle_error_with_location!(self, file_id, reader))?
      {
        Event::Start(e) => {
          let name = e.name();
          let name = reader
            .decode(&name)
            .map_err(handle_error_with_location!(self, file_id, reader))?;

          if name == "Frame" {
            if found_frame {
              panic!("found duplicate frame");
            }

            found_frame = true;
            self.compile_frame(reader, buf, url, file_id)?;
          } else {
            panic!("unknown {}", name);
          }
        }

        Event::End(..) => break,
        Event::Eof => break,

        event => self.handle_event(event, file_id, reader)?,
      }

      buf.clear();
    }

    Ok(())
  }

  fn compile_frame<R: BufRead>(
    &mut self,
    reader: &mut quick_xml::Reader<R>,
    buf: &mut Vec<u8>,
    url: &Url,
    file_id: &FileId,
  ) -> Result<(), ()> {
    buf.clear();

    let mut found_head = false;
    let mut found_body = false;
    loop {
      match reader
        .read_event(buf)
        .map_err(handle_error_with_location!(self, file_id, reader))?
      {
        Event::Start(e) => {
          let name = e.name();
          let name = reader
            .decode(&name)
            .map_err(handle_error_with_location!(self, file_id, reader))?;

          match name {
            "Head" => {
              if found_head {
                panic!("found duplicate head");
              }
              found_head = true;
              self.compile_head(reader, buf, url, file_id)?;
            }

            "Body" => {
              if found_body {
                panic!("found duplicate body");
              }
              found_body = true;
              self.compile_body(reader, buf, url, file_id)?;
            }

            _ => panic!("unknown {}", name),
          }
        }

        Event::End(..) => break,

        event => self.handle_event(event, file_id, reader)?,
      }

      buf.clear();
    }

    if !found_body {
      panic!("found no body");
    }

    Ok(())
  }

  fn compile_head<R: BufRead>(
    &mut self,
    reader: &mut quick_xml::Reader<R>,
    buf: &mut Vec<u8>,
    url: &Url,
    file_id: &FileId,
  ) -> Result<(), ()> {
    buf.clear();

    loop {
      match reader
        .read_event(buf)
        .map_err(handle_error_with_location!(self, file_id, reader))?
      {
        Event::Start(e) => {
          let name = e.name();
          let name = reader
            .decode(&name)
            .map_err(handle_error_with_location!(self, file_id, reader))?;

          match name {
            "Style" => {
              self.compile_style(e.to_owned(), false, reader, buf, url, file_id)?;
            }

            _ => panic!("unknown {}", name),
          }
        }

        Event::Empty(e) => {
          let name = e.name();
          let name = reader
            .decode(&name)
            .map_err(handle_error_with_location!(self, file_id, reader))?;

          match name {
            "Style" => {
              self.compile_style(e.to_owned(), true, reader, buf, url, file_id)?;
            }

            _ => panic!("unknown {}", name),
          }
        }

        Event::End(..) => break,

        event => self.handle_event(event, file_id, reader)?,
      }
      buf.clear();
    }

    Ok(())
  }

  fn compile_body<R: BufRead>(
    &mut self,
    reader: &mut quick_xml::Reader<R>,
    buf: &mut Vec<u8>,
    url: &Url,
    file_id: &FileId,
  ) -> Result<(), ()> {
    buf.clear();

    self.compile_ui_element(self.root, reader, buf, url, file_id)
  }

  fn compile_ui_element<R: BufRead>(
    &mut self,
    parent: NodeId,
    reader: &mut quick_xml::Reader<R>,
    buf: &mut Vec<u8>,
    url: &Url,
    file_id: &FileId,
  ) -> Result<(), ()> {
    buf.clear();

    loop {
      match reader
        .read_event(buf)
        .map_err(handle_error_with_location!(self, file_id, reader))?
      {
        Event::Start(e) => {
          let name = e.name();
          let name = reader
            .decode(&name)
            .map_err(handle_error_with_location!(self, file_id, reader))?;

          match name {
            "Unstyled" => {
              let e = e.to_owned();
              self.compile_unstyled(e, parent, reader, buf, url, file_id)?;
            }

            _ => panic!("unknown {}", name),
          }
        }

        Event::End(..) => break,

        event => self.handle_event(event, file_id, reader)?,
      }

      buf.clear();
    }

    Ok(())
  }

  fn compile_unstyled<'a, R: BufRead>(
    &mut self,
    e: BytesStart<'a>,
    parent: NodeId,
    reader: &mut quick_xml::Reader<R>,
    buf: &mut Vec<u8>,
    url: &Url,
    file_id: &FileId,
  ) -> Result<(), ()> {
    buf.clear();

    let name = e.name();
    let name = reader
      .decode(&name)
      .map_err(handle_error_with_location!(self, file_id, reader))?;

    let mut el = Element::new(ElementData::Unstyled(UnstyledElement));
    for attr in e.attributes() {
      let attr = attr.map_err(handle_error_with_location!(self, file_id, reader))?;
      let key = reader
        .decode(attr.key)
        .map_err(handle_error_with_location!(self, file_id, reader))?;
      let value = attr
        .unescaped_value()
        .map_err(handle_error_with_location!(self, file_id, reader))?;
      let value = reader
        .decode(&value)
        .map_err(handle_error_with_location!(self, file_id, reader))?;

      match key {
        "class" => {
          el.classes = value.split_ascii_whitespace().map(|s| s.to_string()).collect();
        }

        "id" => {
          el.id = Some(value.to_string());
        }

        "style" => {
          unimplemented!();
        }

        _ => {
          self.reporter.add_diagnostic(Diagnostic {
            location: Some((file_id.clone(), reader.buffer_position())),
            min_level: Level::Info,
            kind: DiagnosticKind::InvalidAttribute {
              attr: key.to_string(),
              el: name.to_string(),
            },
          });
        }
      }
    }

    let node = self.body.new_node(el);
    parent.append(node, &mut self.body);

    self.compile_ui_element(node, reader, buf, url, file_id)
  }
}

pub fn compile<URL: IntoUrl, FileId: fmt::Debug + Clone>(
  url: URL,
  reporter: &mut dyn DiagnosticReporter<FileId = FileId>,
) -> Result<CompiledDocument, ()> {
  let url = url.into_url().map_err(handle_error!(reporter))?;

  let mut out = String::new();
  let mut reader = Reader::get(&url).map_err(handle_error!(reporter))?;
  reader.read_to_string(&mut out).map_err(handle_error!(reporter))?;

  let file_id = reporter.add_file(url.to_string(), out);

  let reader = Reader::get(&url).map_err(handle_error!(reporter))?;
  let mut reader = quick_xml::Reader::from_reader(reader);
  reader.check_comments(true);

  let mut buf = Vec::new();

  let mut elements = Arena::new();
  let root = elements.new_node(Element::new(ElementData::Root(RootElement)));

  let mut ctx = Context {
    body: elements,
    root,
    reporter,
    stylesheet: StyleSheet::new(),
  };

  ctx.compile_root(&mut reader, &mut buf, &url, &file_id)?;

  ctx.reporter.checkpoint()?;

  let doc = CompiledDocument {
    version: STRUCTURE_VERSION,
    elements: RwLock::new(ctx.body),
    stylesheet: ctx.stylesheet,
    root,
  };

  doc.init_yoga();

  Ok(doc)
}
