/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// #[cfg(feature="c-event")]
// pub mod c_api;

use std::sync::Arc;
use dom::CompiledDocument;

pub use render::DeviceSize;

pub enum Event {
  Resized(DeviceSize),
  ScaleFactorChanged(f32),
  Redraw,
  Empty,
}

pub trait Windowing {
  fn swap_buffers(&mut self);
  fn make_current(&mut self);
  fn make_not_current(&mut self);
}

pub struct EventHandler<W: Windowing> {
  pub renderer: render::Renderer,
  pub windowing: W,
  pub doc: Arc<CompiledDocument>,
}

impl<W: Windowing> EventHandler<W> {
  #[must_use]
  pub fn new(windowing: W, renderer: render::Renderer, doc: Arc<CompiledDocument>) -> Self {
    Self { windowing, renderer, doc }
  }

  pub fn deinit(mut self) {
    self.windowing.make_current();
    self.renderer.deinit();
    self.windowing.make_not_current();
  }

  pub fn handle_event(&mut self, event: Event) {
    let mut render_inner = false;

    match event {
      Event::Resized(size) => {
        self.renderer.set_device_size(size);
        render_inner = true;
      }

      Event::ScaleFactorChanged(scale) => {
        self.renderer.set_scale_factor(scale);
        render_inner = true;
      }

      Event::Redraw => {
        render_inner = true;
      }

      Event::Empty => {},
    }

    // if self.debug_flags != old_flags {
    //   self.api.send_debug_cmd(DebugCommand::SetFlags(self.debug_flags));
    // }

    self.windowing.make_current();
    self.renderer.render(render_inner, &self.doc);
    self.windowing.swap_buffers();
    self.windowing.make_not_current();
  }
}
