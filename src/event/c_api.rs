use std::os::raw::c_void;

use super::*;

#[doc="module=event"]
pub type EventHandler = super::EventHandler<CWindowing>;
#[doc="module=event"]
pub type EmptyCallback = extern fn(user: *mut c_void);

pub struct CWindowing {
  user: *mut c_void,
  swap_buffers: EmptyCallback,
  make_current: EmptyCallback,
  make_not_current: EmptyCallback,
}

impl Windowing for CWindowing {
  fn swap_buffers(&mut self) {
    (self.swap_buffers)(self.user);
  }

  fn make_current(&mut self) {
    (self.make_current)(self.user);
  }

  fn make_not_current(&mut self) {
    (self.make_not_current)(self.user);
  }
}


#[allow(non_snake_case)]
impl EventHandler {
  #[no_mangle]
  #[doc="module=event,index=0"]
  pub unsafe extern fn EventHandler_new(
    renderer: *mut render::Renderer,
    doc: *const dom::CompiledDocument,
    swap_buffers: EmptyCallback,
    make_current: EmptyCallback,
    make_not_current: EmptyCallback,
    user: *mut c_void,
  ) -> *mut Self {
    let windowing = CWindowing{
      user, swap_buffers, make_current, make_not_current,
    };

    let event_handler = EventHandler::new(windowing, *Box::from_raw(renderer), Arc::from_raw(doc));

    Box::into_raw(Box::new(event_handler))
  }

  #[no_mangle]
  /// This is the brief
  ///
  /// This is the longer description
  #[doc="module=event,index=1"]
  pub unsafe extern fn EventHandler_drop(&mut self) {
    let event_handler = Box::from_raw(self as *mut Self);
    event_handler.deinit();
  }

  #[no_mangle]
  #[doc="module=event,index=2"]
  pub unsafe extern fn EventHandler_handle_resize(&mut self, size: DeviceSize) {
    self.handle_event(Event::Resized(size))
  }

  #[no_mangle]
  #[doc="module=event,index=3"]
  pub unsafe extern fn EventHandler_handle_scale_factor_change(&mut self, scale: f32) {
    self.handle_event(Event::ScaleFactorChanged(scale))
  }

  #[no_mangle]
  #[doc="module=event,index=4"]
  pub unsafe extern fn EventHandler_handle_redraw(&mut self) {
    self.handle_event(Event::Redraw)
  }

  #[no_mangle]
  #[doc="module=event,index=5"]
  pub unsafe extern fn EventHandler_handle_empty(&mut self) {
    self.handle_event(Event::Empty)
  }

  #[no_mangle]
  #[doc="module=event,index=6"]
  pub unsafe extern fn EventHandler_get_user(&mut self) -> *mut c_void {
    self.windowing.user
  }

  #[no_mangle]
  #[doc="module=event,index=7"]
  pub unsafe extern fn EventHandler_set_user(&mut self, user: *mut c_void) {
    self.windowing.user = user;
  }
}
