/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use webrender::{DebugFlags, ShaderPrecacheFlags};
use webrender::api::*;
use webrender::api::units::*;
use euclid::vec2;
use std::rc::Rc;
use gleam::gl::Gl;
use euclid::Size2D;

use std::sync::Arc;
use dom::CompiledDocument;

#[cfg(feature="c-render")]
pub mod c_api;

// pub trait HandyDandyRectBuilder {
//   fn to(&self, x2: i32, y2: i32) -> LayoutRect;
//   fn by(&self, w: i32, h: i32) -> LayoutRect;
// }
// // Allows doing `(x, y).to(x2, y2)` or `(x, y).by(width, height)` with i32
// // values to build a f32 LayoutRect
// impl HandyDandyRectBuilder for (i32, i32) {
//   fn to(&self, x2: i32, y2: i32) -> LayoutRect {
//     LayoutRect::new(
//       LayoutPoint::new(self.0 as f32, self.1 as f32),
//       LayoutSize::new((x2 - self.0) as f32, (y2 - self.1) as f32),
//     )
//   }

//   fn by(&self, w: i32, h: i32) -> LayoutRect {
//     LayoutRect::new(
//       LayoutPoint::new(self.0 as f32, self.1 as f32),
//       LayoutSize::new(w as f32, h as f32),
//     )
//   }
// }

// pub trait Example {
//   const TITLE: &'static str = "WebRender Sample App";
//   const PRECACHE_SHADER_FLAGS: ShaderPrecacheFlags = ShaderPrecacheFlags::EMPTY;
//   const WIDTH: u32 = 1920;
//   const HEIGHT: u32 = 1080;

//   fn render(
//     &mut self,
//     api: &mut RenderApi,
//     builder: &mut DisplayListBuilder,
//     txn: &mut Transaction,
//     device_size: DeviceIntSize,
//     pipeline_id: PipelineId,
//     document_id: DocumentId,
//   );
//   fn on_event(
//     &mut self,
//     _: winit::event::WindowEvent,
//     _: &mut RenderApi,
//     _: DocumentId,
//   ) -> bool {
//     false
//   }
//   fn get_image_handlers(
//     &mut self,
//     _gl: &dyn gl::Gl,
//   ) -> (Option<Box<dyn ExternalImageHandler>>,
//       Option<Box<dyn OutputImageHandler>>) {
//     (None, None)
//   }
//   fn draw_custom(&mut self, _gl: &dyn gl::Gl) {
//   }
// }


const PRECACHE_SHADER_FLAGS: ShaderPrecacheFlags = ShaderPrecacheFlags::EMPTY;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct DevicePixel;

pub type DeviceSize = Size2D<i32, DevicePixel>;

#[doc="module=render"]
pub struct Renderer {
  renderer: webrender::Renderer,
  device_size: DeviceIntSize,
  device_pixel_ratio: f32,
  api: RenderApi,
  pipeline_id: PipelineId,
  document_id: DocumentId,
  layout_size: Size2D<f32, LayoutPixel>,
  epoch: Epoch,
}

impl Renderer {
  pub fn new(gl: Rc<dyn Gl>, device_pixel_ratio: f32, device_size: DeviceSize, notifier: Box<dyn RenderNotifier>) -> Self {
    let device_size = DeviceIntSize::new(device_size.width, device_size.height);
    // let gl = windowing.get_gl();

    // windowing.make_current();

    // info!("OpenGL version {}", gl.get_string(gl::VERSION));

    let debug_flags = DebugFlags::ECHO_DRIVER_MESSAGES;
    let opts = webrender::RendererOptions {
      precache_flags: PRECACHE_SHADER_FLAGS,
      device_pixel_ratio,
      clear_color: Some(ColorF::new(0.3, 0.0, 0.0, 1.0)),
      debug_flags,
      //allow_texture_swizzling: false,
      ..webrender::RendererOptions::default()
    };

    let (renderer, sender) = webrender::Renderer::new(
      gl,
      notifier,
      opts,
      None,
      device_size,
    ).unwrap();
    let mut api = sender.create_api();
    let document_id = api.add_document(device_size, 0);

    // let (external, output) = example.get_image_handlers(&*gl);

    // if let Some(output_image_handler) = output {
    //   renderer.set_output_image_handler(output_image_handler);
    // }

    // if let Some(external_image_handler) = external {
    //   renderer.set_external_image_handler(external_image_handler);
    // }

    let epoch = Epoch(0);
    let pipeline_id = PipelineId(0, 0);
    let layout_size = device_size.to_f32() / euclid::Scale::new(device_pixel_ratio);
    let mut txn = Transaction::new();
    txn.set_root_pipeline(pipeline_id);
    api.send_transaction(document_id, txn);

    Self {
      renderer,
      device_size,
      device_pixel_ratio,
      api,
      pipeline_id,
      document_id,
      layout_size,
      epoch,
    }
  }

  pub fn deinit(self) {
    self.renderer.deinit();
  }

  pub fn set_device_size(&mut self, size: DeviceSize) {
    self.device_size = DeviceIntSize::new(size.width, size.height);
    self.layout_size = self.device_size.to_f32() / euclid::Scale::new(self.device_pixel_ratio);

    let mut txn = Transaction::new();
    txn.set_document_view(self.device_size.into(), self.device_pixel_ratio);
    self.api.send_transaction(self.document_id, txn);
  }

  pub fn set_scale_factor(&mut self, scale: f32) {
    self.device_pixel_ratio = scale;
    self.layout_size = self.device_size.to_f32() / euclid::Scale::new(self.device_pixel_ratio);

    let mut txn = Transaction::new();
    txn.set_document_view(self.device_size.into(), self.device_pixel_ratio);
    self.api.send_transaction(self.document_id, txn);
  }

  pub fn render(&mut self, inner: bool, doc: &Arc<CompiledDocument>) {
    let mut txn = Transaction::new();

    if inner {
      let mut builder = DisplayListBuilder::new(self.pipeline_id, self.layout_size);

      self.render_inner(&mut builder, &mut txn, doc);
      txn.set_display_list(
        self.epoch,
        Some(ColorF::new(0.3, 0.0, 0.0, 1.0)),
        self.layout_size,
        builder.finalize(),
        true,
      );
      txn.generate_frame();
    }

    self.api.send_transaction(self.document_id, txn);

    self.renderer.update();
    self.renderer.render(self.device_size).unwrap();
    let _ = self.renderer.flush_pipeline_info();
  }

  fn render_inner(
    &mut self,
    builder: &mut DisplayListBuilder,
    txn: &mut Transaction,
    doc: &Arc<CompiledDocument>,
  ) {
    let content_bounds = LayoutRect::new(LayoutPoint::zero(), builder.content_size());
    let root_space_and_clip = SpaceAndClipInfo::root_scroll(self.pipeline_id);
    let spatial_id = root_space_and_clip.spatial_id;

    doc.compute_style(self.layout_size.width, self.layout_size.height, yoga::Direction::LTR);
    let arena = doc.elements.write().unwrap();
    for id in doc.root.descendants(&arena) {
      let node = arena[id].get();

      let rect = LayoutRect::new(
        LayoutPoint::new(node.render.left, node.render.top),
        LayoutSize::new(node.render.width, node.render.height),
      );

      builder.push_rect(
        &CommonItemProperties::new(
          rect,
          root_space_and_clip,
        ),
        rect,
        ColorF::new(
          node.render.background_color.0 as f32 / 255.0,
          node.render.background_color.1 as f32 / 255.0,
          node.render.background_color.2 as f32 / 255.0,
          node.render.background_color.3 as f32 / 255.0,
        ),
      );
    }

    // let mask_clip_id = builder.define_clip_image_mask(
    //   &root_space_and_clip,
    //   mask,
    // );
    // let clip_id = builder.define_clip_rounded_rect(
    //   &SpaceAndClipInfo {
    //     spatial_id: root_space_and_clip.spatial_id,
    //     clip_id: mask_clip_id,
    //   },
    //   complex,
    // );

    // builder.push_rect(
    //   &CommonItemProperties::new(
    //     (100, 100).to(200, 200),
    //     root_space_and_clip,
    //   ),
    //   (100, 100).to(200, 200),
    //   ColorF::new(0.0, 1.0, 0.0, 1.0),
    // );

    //  builder.push_rect(
    //   &CommonItemProperties::new(
    //     (250, 100).to(350, 200),
    //     root_space_and_clip,
    //   ),
    //   (250, 100).to(350, 200),
    //   ColorF::new(0.0, 1.0, 0.0, 1.0),
    // );
    // let border_side = BorderSide {
    //   color: ColorF::new(0.0, 0.0, 1.0, 1.0),
    //   style: BorderStyle::Groove,
    // };
    // let border_widths = LayoutSideOffsets::new_all_same(10.0);
    // let border_details = BorderDetails::Normal(NormalBorder {
    //   top: border_side,
    //   right: border_side,
    //   bottom: border_side,
    //   left: border_side,
    //   radius: BorderRadius::uniform(0.0),
    //   do_aa: true,
    // });

    // let bounds = (100, 100).to(200, 200);
    // builder.push_border(
    //   &CommonItemProperties::new(
    //     bounds,
    //     root_space_and_clip,
    //   ),
    //   bounds,
    //   border_widths,
    //   border_details,
    // );

    // builder.push_simple_stacking_context(
    //   content_bounds.origin,
    //   spatial_id,
    //   PrimitiveFlags::IS_BACKFACE_VISIBLE,
    // );

    // let image_mask_key = self.api.generate_image_key();
    // txn.add_image(
    //   image_mask_key,
    //   ImageDescriptor::new(2, 2, ImageFormat::R8, ImageDescriptorFlags::IS_OPAQUE),
    //   ImageData::new(vec![0, 80, 180, 255]),
    //   None,
    // );
    // let mask = ImageMask {
    //   image: image_mask_key,
    //   rect: (75, 75).by(100, 100),
    //   repeat: false,
    // };
    // let complex = ComplexClipRegion::new(
    //   (50, 50).to(150, 150),
    //   BorderRadius::uniform(20.0),
    //   ClipMode::Clip
    // );
    // let mask_clip_id = builder.define_clip_image_mask(
    //   &root_space_and_clip,
    //   mask,
    // );
    // let clip_id = builder.define_clip_rounded_rect(
    //   &SpaceAndClipInfo {
    //     spatial_id: root_space_and_clip.spatial_id,
    //     clip_id: mask_clip_id,
    //   },
    //   complex,
    // );

    // builder.push_rect(
    //   &CommonItemProperties::new(
    //     (100, 100).to(200, 200),
    //     SpaceAndClipInfo { spatial_id, clip_id },
    //   ),
    //   (100, 100).to(200, 200),
    //   ColorF::new(0.0, 1.0, 0.0, 1.0),
    // );

    // builder.push_rect(
    //   &CommonItemProperties::new(
    //     (250, 100).to(350, 200),
    //     SpaceAndClipInfo { spatial_id, clip_id },
    //   ),
    //   (250, 100).to(350, 200),
    //   ColorF::new(0.0, 1.0, 0.0, 1.0),
    // );
    // let border_side = BorderSide {
    //   color: ColorF::new(0.0, 0.0, 1.0, 1.0),
    //   style: BorderStyle::Groove,
    // };
    // let border_widths = LayoutSideOffsets::new_all_same(10.0);
    // let border_details = BorderDetails::Normal(NormalBorder {
    //   top: border_side,
    //   right: border_side,
    //   bottom: border_side,
    //   left: border_side,
    //   radius: BorderRadius::uniform(20.0),
    //   do_aa: true,
    // });

    // let bounds = (100, 100).to(200, 200);
    // builder.push_border(
    //   &CommonItemProperties::new(
    //     bounds,
    //     SpaceAndClipInfo { spatial_id, clip_id },
    //   ),
    //   bounds,
    //   border_widths,
    //   border_details,
    // );

    // if false {
    //   // draw box shadow?
    //   let simple_box_bounds = (20, 200).by(50, 50);
    //   let offset = vec2(10.0, 10.0);
    //   let color = ColorF::new(1.0, 1.0, 1.0, 1.0);
    //   let blur_radius = 0.0;
    //   let spread_radius = 0.0;
    //   let simple_border_radius = 8.0;
    //   let box_shadow_type = BoxShadowClipMode::Inset;

    //   builder.push_box_shadow(
    //     &CommonItemProperties::new(content_bounds, root_space_and_clip),
    //     simple_box_bounds,
    //     offset,
    //     color,
    //     blur_radius,
    //     spread_radius,
    //     BorderRadius::uniform(simple_border_radius),
    //     box_shadow_type,
    //   );
    // }

    // builder.pop_stacking_context();
  }
}
