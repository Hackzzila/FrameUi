use std::sync::Arc;
use dom::CompiledDocument;

use glutin::{
  ContextWrapper,
  NotCurrent,
  PossiblyCurrent,
  ContextBuilder,
  GlRequest,
  window::{WindowId, WindowBuilder},
  event_loop::{EventLoopProxy, EventLoopWindowTarget},
};

use gleam::gl;
use webrender::api::*;

pub use glutin;

pub struct Notifier {
  events_proxy: EventLoopProxy<()>,
}

impl Notifier {
  pub fn new(events_proxy: EventLoopProxy<()>) -> Notifier {
    Notifier { events_proxy }
  }
}

impl RenderNotifier for Notifier {
  fn clone(&self) -> Box<dyn RenderNotifier> {
    Box::new(Notifier {
      events_proxy: self.events_proxy.clone(),
    })
  }

  fn wake_up(&self) {
    #[cfg(not(target_os = "android"))]
    // let _ = self.events_proxy.wakeup();
    let _ = self.events_proxy.send_event(());
  }

  fn new_frame_ready(
    &self,
    _: DocumentId,
    _scrolled: bool,
    _composite_needed: bool,
    _render_time: Option<u64>
  ) {
    self.wake_up();
  }
}

pub struct Window {
  window_id: WindowId,
  event_handler: event::EventHandler<InternalWindow>,
}

impl Window {
  pub fn new<TE>(wb: WindowBuilder, el: &EventLoopWindowTarget<TE>, notifier: Box<dyn RenderNotifier>, doc: Arc<CompiledDocument>) -> Self {
    let windowed_context = ContextBuilder::new()
      .with_gl(GlRequest::GlThenGles {
        opengl_version: (3, 2),
        opengles_version: (3, 0),
      })
      .build_windowed(wb, el)
      .unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let window_id = windowed_context.window().id();

    let device_pixel_ratio = windowed_context.window().scale_factor() as f32;

    let device_size = {
      let size = windowed_context
        .window()
        .inner_size();
      render::DeviceSize::new(size.width as i32, size.height as i32)
    };

    // let notifier = Box::new(Notifier::new(self.proxy.clone()));

    let gl = match windowed_context.get_api() {
      glutin::Api::OpenGl => unsafe {
        gl::GlFns::load_with(
          |symbol| windowed_context.get_proc_address(symbol) as *const _
        )
      },
      glutin::Api::OpenGlEs => unsafe {
        gl::GlesFns::load_with(
          |symbol| windowed_context.get_proc_address(symbol) as *const _
        )
      },
      glutin::Api::WebGl => unimplemented!(),
    };

    use event::Windowing;
    let mut windowing_impl = InternalWindow { windowed_context: GlContext::PossiblyCurrent(windowed_context) };
    windowing_impl.make_current();

    let renderer = render::Renderer::new(gl, device_pixel_ratio, device_size, notifier);

    Self {
      window_id,
      event_handler: event::EventHandler::new(windowing_impl, renderer, doc),
      // renderer: render::Renderer::new(windowing_impl, gl, device_pixel_ratio, device_size, notifier),
    }
  }

  pub fn handle_event<TE: std::fmt::Debug>(&mut self, event: &glutin::event::Event<TE>) {
    // println!("{:?}", event);

    match event {
      glutin::event::Event::WindowEvent { window_id, event } => {
        if *window_id != self.window_id {
          return;
        }

        let event = match event {
          glutin::event::WindowEvent::Resized(size) => {
            event::Event::Resized(render::DeviceSize::new(size.width as i32, size.height as i32))
          }

          glutin::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
            event::Event::ScaleFactorChanged(*scale_factor as f32)
          }

          glutin::event::WindowEvent::AxisMotion { .. } |
          glutin::event::WindowEvent::CursorMoved { .. } => {
            return;
          }

          _ => event::Event::Empty,
        };

        self.event_handler.handle_event(event);
      }

      glutin::event::Event::RedrawRequested(window_id) => {
        if *window_id != self.window_id {
          return;
        }

        self.event_handler.handle_event(event::Event::Redraw);
      }

      _ => {},
    };
  }

  pub fn deinit(self) {
    self.event_handler.deinit();
  }

  pub fn window(&self) -> &glutin::window::Window {
    self.event_handler.windowing.window()
  }
}

enum GlContext {
  PossiblyCurrent(ContextWrapper<PossiblyCurrent, glutin::window::Window>),
  NotCurrent(ContextWrapper<NotCurrent, glutin::window::Window>),
  Empty,
}

struct InternalWindow {
  windowed_context: GlContext,
}

impl InternalWindow {
  fn window(&self) -> &glutin::window::Window {
    match &self.windowed_context {
      GlContext::PossiblyCurrent(ctx) => ctx.window(),
      GlContext::NotCurrent(ctx) => ctx.window(),
      GlContext::Empty => panic!("window called with an empty context"),
    }
  }
}

impl event::Windowing for InternalWindow {
  fn swap_buffers(&mut self) {
    match &self.windowed_context {
      GlContext::PossiblyCurrent(ctx) => ctx.swap_buffers().unwrap(),
      GlContext::NotCurrent(..) => panic!("swap_buffers called with a non-current context"),
      GlContext::Empty => panic!("swap_buffers called with an empty context"),
    }
  }

  fn make_current(&mut self) {
    let ctx = std::mem::replace(&mut self.windowed_context, GlContext::Empty);

    let ctx = unsafe {
      match ctx {
        GlContext::PossiblyCurrent(ctx) => ctx.make_current().unwrap(),
        GlContext::NotCurrent(ctx) => ctx.make_current().unwrap(),
        GlContext::Empty => panic!("make_current called with an empty context"),
      }
    };

    self.windowed_context = GlContext::PossiblyCurrent(ctx);
  }

  fn make_not_current(&mut self) {
    let ctx = std::mem::replace(&mut self.windowed_context, GlContext::Empty);

    let ctx = unsafe {
      match ctx {
        GlContext::PossiblyCurrent(ctx) => ctx.make_not_current().unwrap(),
        GlContext::NotCurrent(ctx) => ctx.make_not_current().unwrap(),
        GlContext::Empty => panic!("make_not_current called with an empty context"),
      }
    };

    self.windowed_context = GlContext::NotCurrent(ctx);
  }
}
