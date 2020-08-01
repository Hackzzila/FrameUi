#define CF_SWIFT_NAME(_name)

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include "generated.h"

#define STRUCTURE_VERSION 0

typedef struct EventHandler_CWindowing EventHandler_CWindowing;

#if defined(MODULE_RENDER)
/**
 *module=render
 */
typedef struct Gl Gl;
#endif

/**
 *module=render
 */
typedef struct Renderer Renderer;

#if defined(MODULE_EVENT)
/**
 *module=event
 */
typedef EventHandler_CWindowing EventHandler;
#endif

/**
 * A 2d size tagged with a unit.
 */
typedef struct {
  /**
   * The extent of the element in the `U` units along the `x` axis (usually horizontal).
   */
  int32_t width;
  /**
   * The extent of the element in the `U` units along the `y` axis (usually vertical).
   */
  int32_t height;
} Size2D_i32__DevicePixel;

typedef Size2D_i32__DevicePixel DeviceSize;

#if defined(MODULE_EVENT)
/**
 *module=event
 */
typedef void (*EmptyCallback)(void *user);
#endif

#if defined(MODULE_RENDER)
/**
 *module=render
 */
typedef const void *(*GlLoadFunc)(const char *symbol);
#endif

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

#if defined(MODULE_EVENT)
/**
 * This is the brief
 *
 * This is the longer description
 *module=event,index=1
 */
void EventHandler_drop(EventHandler *self) CF_SWIFT_NAME(EventHandler.drop(self:));
#endif

#if defined(MODULE_EVENT)
/**
 *module=event,index=6
 */
void *EventHandler_get_user(EventHandler *self) CF_SWIFT_NAME(EventHandler.get_user(self:));
#endif

#if defined(MODULE_EVENT)
/**
 *module=event,index=5
 */
void EventHandler_handle_empty(EventHandler *self) CF_SWIFT_NAME(EventHandler.handle_empty(self:));
#endif

#if defined(MODULE_EVENT)
/**
 *module=event,index=4
 */
void EventHandler_handle_redraw(EventHandler *self) CF_SWIFT_NAME(EventHandler.handle_redraw(self:));
#endif

#if defined(MODULE_EVENT)
/**
 *module=event,index=2
 */
void EventHandler_handle_resize(EventHandler *self,
                                DeviceSize size) CF_SWIFT_NAME(EventHandler.handle_resize(self:size:));
#endif

#if defined(MODULE_EVENT)
/**
 *module=event,index=3
 */
void EventHandler_handle_scale_factor_change(EventHandler *self,
                                             float scale) CF_SWIFT_NAME(EventHandler.handle_scale_factor_change(self:scale:));
#endif

#if defined(MODULE_EVENT)
/**
 *module=event,index=0
 */
EventHandler *EventHandler_new(Renderer *renderer,
                               EmptyCallback swap_buffers,
                               EmptyCallback make_current,
                               EmptyCallback make_not_current,
                               void *user) CF_SWIFT_NAME(EventHandler.new(renderer:swap_buffers:make_current:make_not_current:user:));
#endif

#if defined(MODULE_EVENT)
/**
 *module=event,index=7
 */
void EventHandler_set_user(EventHandler *self,
                           void *user) CF_SWIFT_NAME(EventHandler.set_user(self:user:));
#endif

#if defined(MODULE_RENDER)
/**
 *module=render,index=0
 */
Gl *Gl_load_gl(GlLoadFunc func) CF_SWIFT_NAME(Gl.load_gl(func:));
#endif

#if defined(MODULE_RENDER)
/**
 *module=render,index=1
 */
Gl *Gl_load_gles(GlLoadFunc func) CF_SWIFT_NAME(Gl.load_gles(func:));
#endif

#if defined(MODULE_RENDER)
/**
 *module=render,index=1
 */
void Renderer_drop(Renderer *self) CF_SWIFT_NAME(Renderer.drop(self:));
#endif

#if defined(MODULE_RENDER)
/**
 *module=render,index=0
 */
Renderer *Renderer_new(Gl *gl,
                       float device_pixel_ratio,
                       DeviceSize device_size) CF_SWIFT_NAME(Renderer.new(gl:device_pixel_ratio:device_size:));
#endif

#if defined(MODULE_RENDER)
/**
 *module=render,index=4
 */
void Renderer_render(Renderer *self, bool inner) CF_SWIFT_NAME(Renderer.render(self:inner:));
#endif

#if defined(MODULE_RENDER)
/**
 *module=render,index=2
 */
void Renderer_set_device_size(Renderer *self,
                              DeviceSize size) CF_SWIFT_NAME(Renderer.set_device_size(self:size:));
#endif

#if defined(MODULE_RENDER)
/**
 *module=render,index=3
 */
void Renderer_set_scale_factor(Renderer *self,
                               float scale) CF_SWIFT_NAME(Renderer.set_scale_factor(self:scale:));
#endif

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
