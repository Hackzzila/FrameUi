use std::{
  fmt,
  io::prelude::*,
};

use url::Url;
use quick_xml::events::BytesStart;
use serde::{Serialize, Deserialize};
use source_map_mappings::{Bias, Mappings, parse_mappings};

use style::StyleSheet;

use super::{
  Context,
  Level,
  Reader,
  Diagnostic,
  DiagnosticKind,
  handle_error_with_location,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum StyleType {
  CSS,
  Sass,
  SCSS,
}

#[derive(Debug, Clone)]
enum StyleSource {
  Url(Url),
  Data(String),
}

enum SourceMapOrFileId<FileId> {
  SourceMap(SourceMap),
  FileId(FileId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSourceMap {
  version: u64,
  file: String,
  sources: Vec<String>,
  sources_content: Vec<String>,
  names: Vec<String>,
  mappings: String,
}

#[derive(Debug)]
struct SourceMap {
  version: u64,
  file: String,
  sources: Vec<String>,
  sources_content: Vec<String>,
  names: Vec<String>,
  mappings: Mappings,
}

impl SourceMap {
  fn parse(source: &str) -> SourceMap {
    let source_map: RawSourceMap = serde_json::from_str(source).unwrap();
    SourceMap {
      version: source_map.version,
      file: source_map.file,
      sources: source_map.sources,
      sources_content: source_map.sources_content,
      names: source_map.names,
      mappings: parse_mappings::<()>(source_map.mappings.as_bytes()).unwrap(),
    }
  }
}

impl<'r, FileId: fmt::Debug + Clone> Context<'r, FileId> {
  pub fn compile_style<'a, R: BufRead>(&mut self, e: BytesStart<'a>, empty: bool, reader: &mut quick_xml::Reader<R>, buf: &mut Vec<u8>, url: &Url, file_id: &FileId) -> Result<(), ()> {
    buf.clear();

    let offset = self.reporter.get_line(&file_id, reader.buffer_position());

    let (source, ty) = if empty {
      let mut src = None;
      let mut ty = None;
      for attr in e.attributes() {
        let attr = attr.map_err(handle_error_with_location!(self, file_id, reader))?;
        let key = reader.decode(attr.key).map_err(handle_error_with_location!(self, file_id, reader))?;
        let value = attr.unescaped_value().map_err(handle_error_with_location!(self, file_id, reader))?;
        let value = reader.decode(&value).map_err(handle_error_with_location!(self, file_id, reader))?;

        match key {
          "src" => {
            src = Some(value.to_string());
          }

          "type" => {
            ty = Some(match value.to_lowercase().as_str() {
              "css" => StyleType::CSS,
              "sass" => StyleType::Sass,
              "scss" => StyleType::SCSS,
              _ => unimplemented!(),
            })
          }

          _ => {
            self.reporter.add_diagnostic(Diagnostic {
              location: Some((file_id.clone(), reader.buffer_position())),
              min_level: Level::Info,
              kind: DiagnosticKind::InvalidAttribute {
                attr: key.to_string(),
                el: "Style".to_string(),
              },
            });
          }
        }
      }

      let src = src.unwrap();
      let url = url.join(&src).map_err(handle_error_with_location!(self, file_id, reader))?;

      let ty = ty.unwrap_or_else(|| {
        let filename = url.path_segments().unwrap().next_back().unwrap();
        let mut split = filename.split('.');
        let ext = split.next_back().unwrap();
        if split.next_back().is_some() {
          match ext {
            "css" => StyleType::CSS,
            "sass" => StyleType::Sass,
            _ => StyleType::SCSS,
          }
        } else {
          StyleType::SCSS
        }
      });

      (StyleSource::Url(url), ty)
    } else {
      let mut ty = None;
      for attr in e.attributes() {
        let attr = attr.map_err(handle_error_with_location!(self, file_id, reader))?;
        let key = reader.decode(attr.key).map_err(handle_error_with_location!(self, file_id, reader))?;
        let value = attr.unescaped_value().map_err(handle_error_with_location!(self, file_id, reader))?;
        let value = reader.decode(&value).map_err(handle_error_with_location!(self, file_id, reader))?;

        match key {
          "type" => {
            ty = Some(match value.to_lowercase().as_str() {
              "css" => StyleType::CSS,
              "sass" => StyleType::Sass,
              "scss" => StyleType::SCSS,
              _ => unimplemented!(),
            })
          }

          _ => {
            self.reporter.add_diagnostic(Diagnostic {
              location: Some((file_id.clone(), reader.buffer_position())),
              min_level: Level::Info,
              kind: DiagnosticKind::InvalidAttribute {
                attr: key.to_string(),
                el: "Style".to_string(),
              },
            });
          }
        }
      }

      let text = reader.read_text(e.name(), buf).map_err(handle_error_with_location!(self, file_id, reader))?;
      (StyleSource::Data(text), ty.unwrap_or(StyleType::SCSS))
    };

    let (css, offset, source) = match ty {
      StyleType::CSS => {
        match source {
          StyleSource::Url(url) => {
            let mut url_reader = Reader::get(&url).map_err(handle_error_with_location!(self, file_id, reader))?;
            let mut out = String::new();
            url_reader.read_to_string(&mut out).map_err(handle_error_with_location!(self, file_id, reader))?;
            let file_id = self.reporter.add_file(url.to_string(), out.clone());
            (out, 0, SourceMapOrFileId::FileId(file_id))
          }

          StyleSource::Data(text) => {
            (text, offset, SourceMapOrFileId::FileId(file_id.clone()))
          }
        }
      }

      ty => {
        let (text, url) = match source {
          StyleSource::Url(url) => {
            let mut url_reader = Reader::get(&url).map_err(handle_error_with_location!(self, file_id, reader))?;

            let mut out = String::new();
            url_reader.read_to_string(&mut out).map_err(handle_error_with_location!(self, file_id, reader))?;

            (out, url)
          }

          StyleSource::Data(text) => {
            (text, Url::parse("file:///C/bar.txt").unwrap())
          }
        };

        let ctx = sass::DataContext::new(&text).unwrap();
        let opt = ctx.options();
        opt.set_input_path(url.as_str()).unwrap();
        opt.set_source_map_file("stdin").unwrap();
        opt.set_source_map_contents(true);
        opt.set_is_indented_syntax_src(ty == StyleType::Sass);

        let compiled = ctx.compile().map_err(|e| {
          let file_id = self.reporter.add_file(e.file().unwrap(), e.src().unwrap());
          let pos = self.reporter.get_position(&file_id, e.line() as usize - 1, e.column() as usize);
          self.reporter.add_diagnostic(Diagnostic {
            location: Some((file_id, pos)),
            min_level: Level::Error,
            kind: DiagnosticKind::SassParseError(e.text().unwrap()),
          });
        })?;

        let css = compiled.output().unwrap();
        let source_map = unsafe { compiled.source_map().unwrap() };
        let source_map = SourceMap::parse(&source_map);

        (css, 0, SourceMapOrFileId::SourceMap(source_map))
      }
    };

    let mut input = StyleSheet::create_parser_input_with_line_offset(&css, offset as u32);
    self.stylesheet.parse(&mut input).map_err(|e| {
      let location = match source {
        SourceMapOrFileId::FileId(file_id) => {
          let pos = self.reporter.get_position(&file_id, e.0.location.line as usize, e.0.location.column as usize);
          Some((file_id, pos))
        }

        SourceMapOrFileId::SourceMap(source_map) => {
          let original_location = source_map.mappings.original_location_for(e.0.location.line, e.0.location.column, Bias::GreatestLowerBound).unwrap();
          let original_location = original_location.original.as_ref().unwrap();

          let file_id = self.reporter.add_file(
            source_map.sources[original_location.source as usize].clone(),
            source_map.sources_content[original_location.source as usize].clone()
          );

          let pos = self.reporter.get_position(&file_id, original_location.original_line as usize, original_location.original_column as usize);

          Some((file_id, pos))
        }
      };

      self.reporter.add_diagnostic(Diagnostic {
        location,
        min_level: Level::Error,
        kind: DiagnosticKind::CssParseError(e),
      });
    })?;

    Ok(())
  }
}
