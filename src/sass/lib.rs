pub mod sys {
  #![allow(non_upper_case_globals)]
  #![allow(non_camel_case_types)]
  #![allow(non_snake_case)]

  include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::str::Utf8Error;
use std::ffi::{CStr, CString, NulError};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct FileContext {
  file_ctx: *mut sys::Sass_File_Context,
  ctx: *mut sys::Sass_Context,
}

impl FileContext {
  pub fn new(input_path: &str) -> Result<Self, NulError> {
    let input_path = CString::new(input_path)?;
    unsafe {
      let file_ctx = sys::sass_make_file_context(input_path.as_ptr());
      Ok(Self {
        file_ctx,
        ctx: sys::sass_file_context_get_context(file_ctx),
      })
    }
  }

  pub fn options<'ctx>(&'ctx self) -> Options<'ctx> {
    unsafe {
      Options {
        opts: sys::sass_context_get_options(self.ctx),
        _phantom: PhantomData,
      }
    }
  }

  pub fn compile<'ctx>(&'ctx self) -> Result<Compiled<'ctx>, Error<'ctx>> {
    unsafe {
      let status = sys::sass_compile_file_context(self.file_ctx);

      if status == 0 {
        Ok(Compiled {
          ctx: self.ctx,
          _phantom: PhantomData,
        })
      } else {
        Err(Error {
          ctx: self.ctx,
          _phantom: PhantomData,
        })
      }
    }
  }
}

impl Drop for FileContext {
  fn drop(&mut self) {
    unsafe {
      sys::sass_delete_file_context(self.file_ctx);
    }
  }
}

#[derive(Debug)]
pub struct DataContext {
  data_ctx: *mut sys::Sass_Data_Context,
  ctx: *mut sys::Sass_Context,
}

impl DataContext {
  pub fn new(source_string: &str) -> Result<Self, NulError> {
    let source_string = CString::new(source_string)?;
    unsafe {
      let data_ctx = sys::sass_make_data_context(sys::sass_copy_c_string(source_string.as_ptr()));
      Ok(Self {
        data_ctx,
        ctx: sys::sass_data_context_get_context(data_ctx),
      })
    }
  }

  pub fn options<'ctx>(&'ctx self) -> Options<'ctx> {
    unsafe {
      Options {
        opts: sys::sass_context_get_options(self.ctx),
        _phantom: PhantomData,
      }
    }
  }

  pub fn compile<'ctx>(&'ctx self) -> Result<Compiled<'ctx>, Error<'ctx>> {
    unsafe {
      let status = sys::sass_compile_data_context(self.data_ctx);

      if status == 0 {
        Ok(Compiled {
          ctx: self.ctx,
          _phantom: PhantomData,
        })
      } else {
        Err(Error {
          ctx: self.ctx,
          _phantom: PhantomData,
        })
      }
    }
  }
}

impl Drop for DataContext {
  fn drop(&mut self) {
    unsafe {
      sys::sass_delete_data_context(self.data_ctx);
    }
  }
}

#[derive(Debug)]
pub struct Options<'ctx> {
  opts: *mut sys::Sass_Options,
  _phantom: PhantomData<&'ctx ()>,
}

impl Options<'_> {
  pub fn set_source_map_file(&self, value: &str) -> Result<(), NulError> {
    unsafe {
      let value = CString::new(value)?;
      sys::sass_option_set_source_map_file(self.opts, value.as_ptr());
      Ok(())
    }
  }

  pub fn set_source_map_contents(&self, value: bool) {
    unsafe {
      sys::sass_option_set_source_map_contents(self.opts, value);
    }
  }

  pub fn set_is_indented_syntax_src(&self, value: bool) {
    unsafe {
      sys::sass_option_set_is_indented_syntax_src(self.opts, value);
    }
  }

  pub fn set_input_path(&self, value: &str) -> Result<(), NulError> {
    unsafe {
      let value = CString::new(value)?;
      sys::sass_option_set_input_path(self.opts, value.as_ptr());
      Ok(())
    }
  }
}

#[derive(Debug)]
pub struct Compiled<'ctx> {
  ctx: *mut sys::Sass_Context,
  _phantom: PhantomData<&'ctx ()>,
}

impl Compiled<'_> {
  pub fn status(&self) -> i32 {
    unsafe {
      sys::sass_context_get_error_status(self.ctx)
    }
  }

  pub fn output(&self) -> Result<String, Utf8Error> {
    unsafe {
      let c_str = CStr::from_ptr(sys::sass_context_get_output_string(self.ctx));
      Ok(c_str.to_str()?.to_string())
    }
  }

  pub unsafe fn source_map(&self) -> Result<String, Utf8Error> {
    let c_str = CStr::from_ptr(sys::sass_context_get_source_map_string(self.ctx));
    Ok(c_str.to_str()?.to_string())
  }
}

#[derive(Debug)]
pub struct Error<'ctx> {
  ctx: *mut sys::Sass_Context,
  _phantom: PhantomData<&'ctx ()>,
}

impl Error<'_> {
  pub fn status(&self) -> i32 {
    unsafe {
      sys::sass_context_get_error_status(self.ctx)
    }
  }

  pub fn json(&self) -> Result<String, Utf8Error> {
    unsafe {
      let c_str = CStr::from_ptr(sys::sass_context_get_error_json(self.ctx));
      Ok(c_str.to_str()?.to_string())
    }
  }

  pub fn text(&self) -> Result<String, Utf8Error> {
    unsafe {
      let c_str = CStr::from_ptr(sys::sass_context_get_error_text(self.ctx));
      Ok(c_str.to_str()?.to_string())
    }
  }

  pub fn message(&self) -> Result<String, Utf8Error> {
    unsafe {
      let c_str = CStr::from_ptr(sys::sass_context_get_error_message(self.ctx));
      Ok(c_str.to_str()?.to_string())
    }
  }

  pub fn file(&self) -> Result<String, Utf8Error> {
    unsafe {
      let c_str = CStr::from_ptr(sys::sass_context_get_error_file(self.ctx));
      Ok(c_str.to_str()?.to_string())
    }
  }

  pub fn src(&self) -> Result<String, Utf8Error> {
    unsafe {
      let c_str = CStr::from_ptr(sys::sass_context_get_error_src(self.ctx));
      Ok(c_str.to_str()?.to_string())
    }
  }

  pub fn line(&self) -> u64 {
    unsafe {
      sys::sass_context_get_error_line(self.ctx)
    }
  }

  pub fn column(&self) -> u64 {
    unsafe {
      sys::sass_context_get_error_column(self.ctx)
    }
  }
}
