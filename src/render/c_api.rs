#![allow(non_snake_case)]

use std::{
  ffi::CString,
  os::raw::{c_char, c_void},
};

use gleam::gl;

use super::*;

#[repr(C)]
#[doc = "module=render"]
pub struct DeviceSize {
  pub width: i32,
  pub height: i32,
}

impl Into<super::DeviceSize> for DeviceSize {
  fn into(self) -> super::DeviceSize {
    super::DeviceSize::new(self.width, self.height)
  }
}

#[doc = "module=render"]
pub struct Gl;

#[doc = "module=render"]
pub type GlLoadFunc = extern "C" fn(symbol: *const c_char) -> *const c_void;

impl Gl {
  #[no_mangle]
  #[doc = "module=render,index=0"]
  pub unsafe extern "C" fn Gl_load_gl(func: GlLoadFunc) -> *mut Gl {
    let gl = gl::GlFns::load_with(|symbol| {
      let string = CString::new(symbol).unwrap();
      func(string.as_ptr())
    });

    Box::into_raw(Box::new(gl)) as *mut _
  }

  #[no_mangle]
  #[doc = "module=render,index=1"]
  pub unsafe extern "C" fn Gl_load_gles(func: GlLoadFunc) -> *mut Gl {
    let gl = gl::GlesFns::load_with(|symbol| {
      let string = CString::new(symbol).unwrap();
      func(string.as_ptr())
    });

    Box::into_raw(Box::new(gl)) as *mut _
  }
}

pub struct Notifier;

impl RenderNotifier for Notifier {
  fn clone(&self) -> Box<dyn RenderNotifier> {
    Box::new(Notifier)
  }

  fn wake_up(&self) {
    // #[cfg(not(target_os = "android"))]
    // let _ = self.events_proxy.wakeup();
    // let _ = self.events_proxy.send_event(());
    // panic!("foo");
  }

  fn new_frame_ready(&self, _: DocumentId, _scrolled: bool, _composite_needed: bool, _render_time: Option<u64>) {
    // self.wake_up();
    // panic!("bar");
  }
}

// #[doc="module=render"]
// pub type Renderer = super::Renderer;

#[allow(non_snake_case)]
impl Renderer {
  #[no_mangle]
  #[doc = "module=render,index=0"]
  pub unsafe extern "C" fn Renderer_new(gl: *mut Gl, device_pixel_ratio: f32, device_size: DeviceSize) -> *mut Self {
    let gl = *Box::from_raw(gl as *mut _);

    let renderer = Renderer::new(gl, device_pixel_ratio, device_size.into(), Box::new(Notifier));

    Box::into_raw(Box::new(renderer))
  }

  #[no_mangle]
  #[doc = "module=render,index=1"]
  pub unsafe extern "C" fn Renderer_drop(&mut self) {
    let renderer = Box::from_raw(self as *mut Self);
    renderer.deinit();
  }

  #[no_mangle]
  #[doc = "module=render,index=2"]
  pub unsafe extern "C" fn Renderer_set_device_size(&mut self, size: DeviceSize) {
    self.set_device_size(size.into());
  }

  #[no_mangle]
  #[doc = "module=render,index=3"]
  pub unsafe extern "C" fn Renderer_set_scale_factor(&mut self, scale: f32) {
    self.set_scale_factor(scale);
  }

  #[no_mangle]
  #[doc = "module=render,index=4"]
  pub unsafe extern "C" fn Renderer_render(&mut self, inner: bool, doc: *const dom::CompiledDocument) {
    let doc = Arc::from_raw(doc);
    self.render(inner, &doc);
    Arc::into_raw(doc);
  }
}
