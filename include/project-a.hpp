
#include <cassert>

namespace frameui {

namespace c_api {
#include "project-a.h"
}
namespace event {
typedef void (*EmptyCallback)(void *);

class EventHandler {
 public:
  EventHandler(Renderer *renderer, const CompiledDocument *doc,
               EmptyCallback swap_buffers, EmptyCallback make_current,
               EmptyCallback make_not_current, void *user) {
    self = c_api::EventHandler_new(renderer, doc, swap_buffers, make_current,
                                   make_not_current, user);
  }

  ~EventHandler() {
    if (self) {
      c_api::EventHandler_drop(self);
    }
  }

  void HandleResize(DeviceSize size) {
    assert(self != nullptr);
    return c_api::EventHandler_handle_resize(self, size);
  }

  void HandleScaleFactorChange(float scale) {
    assert(self != nullptr);
    return c_api::EventHandler_handle_scale_factor_change(self, scale);
  }

  void HandleRedraw() {
    assert(self != nullptr);
    return c_api::EventHandler_handle_redraw(self);
  }

  void HandleEmpty() {
    assert(self != nullptr);
    return c_api::EventHandler_handle_empty(self);
  }

  void *GetUser() {
    assert(self != nullptr);
    return c_api::EventHandler_get_user(self);
  }

  void SetUser(void *user) {
    assert(self != nullptr);
    return c_api::EventHandler_set_user(self, user);
  }

  c_api::EventHandler *GetInternalPointer() { return self; }

  c_api::EventHandler *TakeInternalPointer() {
    c_api::EventHandler *out = self;
    self = nullptr;
    return out;
  }

 private:
  c_api::EventHandler *self = nullptr;
};

}  // namespace event

namespace render {

class DeviceSize {
 public:
  c_api::DeviceSize *GetInternalPointer() { return self; }

  c_api::DeviceSize *TakeInternalPointer() {
    c_api::DeviceSize *out = self;
    self = nullptr;
    return out;
  }

 private:
  c_api::DeviceSize *self = nullptr;
};

class Gl {
 public:
  static Gl *LoadGl(GlLoadFunc func) {
    assert(self != nullptr);
    return c_api::Gl_load_gl(func);
  }

  static Gl *LoadGles(GlLoadFunc func) {
    assert(self != nullptr);
    return c_api::Gl_load_gles(func);
  }

  c_api::Gl *GetInternalPointer() { return self; }

  c_api::Gl *TakeInternalPointer() {
    c_api::Gl *out = self;
    self = nullptr;
    return out;
  }

 private:
  c_api::Gl *self = nullptr;
};

typedef const void *(*GlLoadFunc)(const char *);

class Renderer {
 public:
  Renderer(Gl *gl, float device_pixel_ratio, DeviceSize device_size) {
    self = c_api::Renderer_new(gl, device_pixel_ratio, device_size);
  }

  ~Renderer() {
    if (self) {
      c_api::Renderer_drop(self);
    }
  }

  void SetDeviceSize(DeviceSize size) {
    assert(self != nullptr);
    return c_api::Renderer_set_device_size(self, size);
  }

  void SetScaleFactor(float scale) {
    assert(self != nullptr);
    return c_api::Renderer_set_scale_factor(self, scale);
  }

  void Render(_Bool inner, const CompiledDocument *doc) {
    assert(self != nullptr);
    return c_api::Renderer_render(self, inner, doc);
  }

  c_api::Renderer *GetInternalPointer() { return self; }

  c_api::Renderer *TakeInternalPointer() {
    c_api::Renderer *out = self;
    self = nullptr;
    return out;
  }

 private:
  c_api::Renderer *self = nullptr;
};

}  // namespace render
}  // namespace frameui