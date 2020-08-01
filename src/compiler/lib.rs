use std::{
  io,
  io::BufReader,
  fs::File,
  fmt,
  error,
  path::Path,
  sync::{
    RwLock,
  },
  io::prelude::*,
};

use url::{Url};
use indextree::{Arena, NodeId};
use quick_xml::events::{Event, BytesStart};

use dom::{STRUCTURE_VERSION, Element, ElementData, RootElement, UnstyledElement, CompiledDocument};
use style::StyleSheet;

pub trait IntoUrl<FileId> {
  fn into_url(&self) -> Result<Url, Error<FileId>>;
}

impl<FileId> IntoUrl<FileId> for &str {
  fn into_url(&self) -> Result<Url, Error<FileId>> {
    Ok(Url::parse(self)?)
  }
}

impl<T: AsRef<Path>, FileId> IntoUrl<FileId> for &T {
  fn into_url(&self) -> Result<Url, Error<FileId>> {
    let path = self.as_ref().canonicalize()?;
    Ok(Url::from_file_path(path).unwrap())
  }
}

pub struct Diagnostic<FileId> {
  pub data: DiagnosticData,
  pub pos: usize,
  pub file_id: FileId,
}

impl<FileId> fmt::Display for Diagnostic<FileId> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.data.fmt(f)
  }
}

pub enum DiagnosticData {
  InvalidElement(String),
  InvalidContext(String, String, String),
  InvalidAttribute(String, String),
  ExpectedSelfClosing,
  ExpectedClosingTag(String),
}

impl fmt::Display for DiagnosticData {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InvalidElement(e) => write!(f, "invalid element {}", e),
      Self::InvalidContext(name, ty, parent) => write!(f, "element `{}` of type `{}` is not allowed inside `{}`", name, ty, parent),
      Self::InvalidAttribute(attr, el) => write!(f, "invalid attribute `{}` for `{}`", attr, el),
      Self::ExpectedSelfClosing => write!(f, "childless elements should be self-closing"),
      Self::ExpectedClosingTag(e) => write!(f, "element `{}` should have explicit closing tag", e),
    }
  }
}

#[derive(Debug)]
pub enum Error<FileId> {
  IOError(io::Error),
  ReqwestError(reqwest::Error),
  ParseError(quick_xml::Error, FileId, usize),
  UrlParseError(url::ParseError),
  MissingNode(NodeId, &'static str, u32, u32),
}

impl<FileId> fmt::Display for Error<FileId> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Error::IOError(e) => e.fmt(f),
      Error::ReqwestError(e) => e.fmt(f),
      Error::ParseError(e, ..) => e.fmt(f),
      Error::UrlParseError(e) => e.fmt(f),
      Error::MissingNode(node, file, line, col) => write!(f, "missing node `{}`, {}:{}:{} ", node, file, line, col),
    }
  }
}

impl<FileId: fmt::Debug> error::Error for Error<FileId> {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match self {
      Error::IOError(e) => Some(e),
      Error::ReqwestError(e) => Some(e),
      Error::ParseError(e, ..) => Some(e),
      Error::UrlParseError(e) => Some(e),
      _ => None,
    }
  }
}

impl<FileId> From<io::Error> for Error<FileId> {
  fn from(e: io::Error) -> Error<FileId> {
    Error::IOError(e)
  }
}

impl<FileId> From<reqwest::Error> for Error<FileId> {
  fn from(e: reqwest::Error) -> Error<FileId> {
    Error::ReqwestError(e)
  }
}

impl<FileId> From<url::ParseError> for Error<FileId> {
  fn from(e: url::ParseError) -> Error<FileId> {
    Error::UrlParseError(e)
  }
}

// impl<FileId> From<quick_xml::Error> for Error<FileId> {
//   fn from(e: quick_xml::Error) -> Error<FileId> {
//     Error::ParseError(e)
//   }
// }

use reqwest::blocking::{get, Response};

pub enum Reader {
  File(BufReader<File>),
  Network(BufReader<Response>),
}

impl Reader {
  pub fn get<FileId>(url: &Url) -> Result<Reader, Error<FileId>> {
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
  type FileId;
  fn add_file(&mut self, url: &Url) -> Result<Self::FileId, Error<Self::FileId>>;
  fn add_diagnostic(&mut self, diagnostic: Diagnostic<Self::FileId>) -> Result<(), Error<Self::FileId>>;
}

struct Context<'r, FileId: Clone> {
  body: Arena<dom::Element>,
  root: NodeId,
  reporter: &'r mut dyn DiagnosticReporter<FileId=FileId>,
  stylesheet: StyleSheet,
}

impl<'r, FileId: Clone> Context<'r, FileId> {
  fn compile_root<R: BufRead>(&mut self, reader: &mut quick_xml::Reader<R>, buf: &mut Vec<u8>, url: &Url, file_id: &FileId) -> Result<(), Error<FileId>> {
    buf.clear();

    let mut found_frame = false;
    loop {
      match reader.read_event(buf).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))? {
        Event::Start(e) => {
          let name = e.name();
          let name = reader.decode(&name).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?;

          if name == "Frame" {
            if found_frame == true {
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

        _ => {
          panic!("unexpected at line {}", line!());
        },
      }

      buf.clear();
    }

    Ok(())
  }

  fn compile_frame<R: BufRead>(&mut self, reader: &mut quick_xml::Reader<R>, buf: &mut Vec<u8>, url: &Url, file_id: &FileId) -> Result<(), Error<FileId>> {
    buf.clear();

    let mut found_head = false;
    let mut found_body = false;
    loop {
      match reader.read_event(buf).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))? {
        Event::Start(e) => {
          let name = e.name();
          let name = reader.decode(&name).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?;

          match name {
            "Head" => {
              if found_head == true {
                panic!("found duplicate head");
              }
              found_head = true;
              self.compile_head(reader, buf, url, file_id)?;
            },

            "Body" => {
              if found_body == true {
                panic!("found duplicate body");
              }
              found_body = true;
              self.compile_body(reader, buf, url, file_id)?;
            },

            _ => panic!("unknown {}", name)
          }
        }

        Event::End(..) => break,

        _ => {
          panic!("unexpected at line {}", line!());
        },
      }

      buf.clear();
    }

    if !found_body {
      panic!("found no body");
    }

    Ok(())
  }

  fn compile_head<R: BufRead>(&mut self, reader: &mut quick_xml::Reader<R>, buf: &mut Vec<u8>, url: &Url, file_id: &FileId) -> Result<(), Error<FileId>> {
    buf.clear();

    loop {
      match reader.read_event(buf).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))? {
        Event::Start(e) => {
          let name = e.name();
          let name = reader.decode(&name).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?;

          match name {
            "Style" => {
              self.compile_style(e.to_owned(), false, reader, buf, url, file_id)?;
            },

            _ => panic!("unknown {}", name)
          }
        }

        Event::Empty(e) => {
          let name = e.name();
          let name = reader.decode(&name).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?;

          match name {
            "Style" => {
              self.compile_style(e.to_owned(), true, reader, buf, url, file_id)?;
            },

            _ => panic!("unknown {}", name)
          }
        }

        Event::End(..) => break,

        _ => {
          panic!("unexpected at line {}", line!());
        },
      }
      buf.clear();
    }

    Ok(())
  }

  fn compile_style<'a, R: BufRead>(&mut self, e: BytesStart<'a>, empty: bool, reader: &mut quick_xml::Reader<R>, buf: &mut Vec<u8>, url: &Url, file_id: &FileId) -> Result<(), Error<FileId>> {
    buf.clear();

    let text = if empty {
      let mut src = None;
      for attr in e.attributes() {
        let attr = attr.map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?;
        let key = reader.decode(attr.key).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?;
        let value = attr.unescaped_value().map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?;
        let value = reader.decode(&value).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?;

        match key {
          "src" => {
            src = Some(value.to_string());
          }

          _ => {
            self.reporter.add_diagnostic(Diagnostic {
              pos: reader.buffer_position(),
              file_id: file_id.clone(),
              data: DiagnosticData::InvalidAttribute(
                key.to_string(),
                "Style".to_string(),
              ),
            })?;
          }
        }
      }

      let src = src.unwrap();
      let url = url.join(&src)?;
      let mut reader = Reader::get(&url)?;

      let mut out = String::new();
      reader.read_to_string(&mut out)?;

      out
    } else {
      reader.read_text(e.name(), buf).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?
    };

    self.stylesheet.parse(&text);

    Ok(())
  }

  fn compile_body<R: BufRead>(&mut self, reader: &mut quick_xml::Reader<R>, buf: &mut Vec<u8>, url: &Url, file_id: &FileId) -> Result<(), Error<FileId>> {
    buf.clear();

    self.compile_ui_element(self.root, reader, buf, url, file_id)
  }

  fn compile_ui_element<R: BufRead>(&mut self, parent: NodeId, reader: &mut quick_xml::Reader<R>, buf: &mut Vec<u8>, url: &Url, file_id: &FileId) -> Result<(), Error<FileId>> {
    buf.clear();

    loop {
      match reader.read_event(buf).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))? {
        Event::Start(e) => {
          let name = e.name();
          let name = reader.decode(&name).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?;

          match name {
            "Unstyled" => {
              let e = e.to_owned();
              self.compile_unstyled(e, parent, reader, buf, url, file_id)?;
            }

            _ => panic!("unknown {}", name)
          }
        }

        Event::End(..) => break,

        _ => {
          panic!("unexpected at line {}", line!());
        },
      }

      buf.clear();
    }

    Ok(())
  }

  fn compile_unstyled<'a, R: BufRead>(&mut self, e: BytesStart<'a>, parent: NodeId, reader: &mut quick_xml::Reader<R>, buf: &mut Vec<u8>, url: &Url, file_id: &FileId) -> Result<(), Error<FileId>> {
    buf.clear();


    let name = e.name();
    let name = reader.decode(&name).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?;

    let mut el = Element::new(ElementData::Unstyled(UnstyledElement));
    for attr in e.attributes() {
      let attr = attr.map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?;
      let key = reader.decode(attr.key).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?;
      let value = attr.unescaped_value().map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?;
      let value = reader.decode(&value).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?;

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
            pos: reader.buffer_position(),
            file_id: file_id.clone(),
            data: DiagnosticData::InvalidAttribute(
              key.to_string(),
              name.to_string(),
            ),
          })?;
        }
      }
    }

    let node = self.body.new_node(el);
    parent.append(node, &mut self.body);

    self.compile_ui_element(node, reader, buf, url, file_id)
  }
}

pub fn compile<URL: IntoUrl<FileId>, FileId: Clone>(url: URL, reporter: &mut dyn DiagnosticReporter<FileId=FileId>) -> Result<CompiledDocument, Error<FileId>> {
  let url = url.into_url()?;

  let file_id = reporter.add_file(&url)?;

  let reader = Reader::get(&url)?;
  let mut reader = quick_xml::Reader::from_reader(reader);
  reader.trim_text(true);
  // reader.expand_empty_elements(true);
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

  let doc = CompiledDocument {
    version: STRUCTURE_VERSION,
    elements: RwLock::new(ctx.body),
    stylesheet: ctx.stylesheet,
    root,
  };

  doc.init_yoga();

  Ok(doc)

  // compile_root(&mut ctx, &mut reader, &mut buf, reporter, file_id.clone())?;

  // loop {
  //   match reader.read_event(&mut buf).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))? {
  //     Event::Start(e) => {
  //       let name = e.name();
  //       let name = reader.decode(&name).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?;
  //       let el = spec::parse_element!();

  //       let parent = elements.get(current)
  //         .ok_or(Error::MissingNode(current, file!(), line!(), column!()))?
  //         .get();

  //       if !el.is_error() {
  //         if !parent.get_children().contains(&el.get_type()) {
  //           reporter.add_diagnostic(Diagnostic {
  //             pos: reader.buffer_position(),
  //             file_id: file_id.clone(),
  //             data: DiagnosticData::InvalidContext(
  //               el.get_name().to_string(),
  //               el.get_type().to_string(),
  //               parent.get_name().to_string(),
  //             ),
  //           })?;
  //         }

  //         if el.get_children().is_empty() {
  //           reporter.add_diagnostic(Diagnostic {
  //             pos: reader.buffer_position(),
  //             file_id: file_id.clone(),
  //             data: DiagnosticData::ExpectedSelfClosing
  //           })?;
  //         }
  //       }

  //       let node = elements.new_node(el);
  //       current.append(node, &mut elements);
  //       current = node;
  //     }

  //     Event::End(..) => {
  //       current = current.ancestors(&elements)
  //         .nth(1)
  //         .ok_or(Error::MissingNode(current, file!(), line!(), column!()))?;
  //     }

  //     Event::Empty(e) => {
  //       let name = e.name();
  //       let name = reader.decode(&name).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?;
  //       let el = spec::parse_element!();

  //       let parent = elements.get(current)
  //         .ok_or(Error::MissingNode(current, file!(), line!(), column!()))?
  //         .get();

  //       if !el.is_error() {
  //         if !parent.get_children().contains(&el.get_type()) {
  //           reporter.add_diagnostic(Diagnostic {
  //             pos: reader.buffer_position(),
  //             file_id: file_id.clone(),
  //             data: DiagnosticData::InvalidContext(
  //               el.get_name().to_string(),
  //               el.get_type().to_string(),
  //               parent.get_name().to_string(),
  //             ),
  //           })?;
  //         }

  //         if el.get_children().is_empty() {
  //           reporter.add_diagnostic(Diagnostic {
  //             pos: reader.buffer_position(),
  //             file_id: file_id.clone(),
  //             data: DiagnosticData::ExpectedSelfClosing
  //           })?;
  //         }
  //       }

  //       let node = elements.new_node(el);
  //       current.append(node, &mut elements);
  //     }

  //     Event::Text(e) => {
  //       let el = Element::Text(e.unescape_and_decode(&reader).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?);

  //       let parent = elements.get(current)
  //         .ok_or(Error::MissingNode(current, file!(), line!(), column!()))?
  //         .get();

  //       if !parent.get_children().contains(&el.get_type()) {
  //         reporter.add_diagnostic(Diagnostic {
  //           pos: reader.buffer_position(),
  //           file_id: file_id.clone(),
  //           data: DiagnosticData::InvalidContext(
  //             el.get_name().to_string(),
  //             el.get_type().to_string(),
  //             parent.get_name().to_string(),
  //           ),
  //         })?;
  //       }

  //       current.append(elements.new_node(el), &mut elements);
  //     }

  //     // Event::CData(e) => {
  //     //   current.append(elements.new_node(Element::CData(e.unescape_and_decode(&reader)?)), &mut elements);
  //     // }

  //     Event::Comment(e) => {
  //       let el = Element::Comment(e.unescape_and_decode(&reader).map_err(|e| Error::ParseError(e, file_id.clone(), reader.buffer_position()))?);

  //       let parent = elements.get(current)
  //         .ok_or(Error::MissingNode(current, file!(), line!(), column!()))?
  //         .get();

  //       if !parent.get_children().contains(&el.get_type()) {
  //         reporter.add_diagnostic(Diagnostic {
  //           pos: reader.buffer_position(),
  //           file_id: file_id.clone(),
  //           data: DiagnosticData::InvalidContext(
  //             el.get_name().to_string(),
  //             el.get_type().to_string(),
  //             parent.get_name().to_string(),
  //           ),
  //         })?;
  //       }

  //       current.append(elements.new_node(el), &mut elements);
  //     }

  //     // Event::PI(e) => {
  //     //   current.append(elements.new_node(Element::ProcessingInstruction(e.unescape_and_decode(&reader)?)), &mut elements);
  //     // }

  //     Event::Eof => break,
  //     _ => {},
  //   }
  //   buf.clear();
  // }

  // Ok(())

  // let page = Page {
  //   root,
  //   elements: RwLock::new(elements),
  //   devtools: DashMap::new(),
  // };

  // Ok(View {
  //   version: STRUCTURE_VERSION,
  //   pages: vec![page],
  //   current_page: AtomicUsize::new(0),
  // })
}
