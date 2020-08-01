#define MODULE_EVENT
#define MODULE_RENDER

#include <iostream>
#include "project-a.h"
#include <GLFW/glfw3.h>

const void *get_proc_address(const char *symbol) {
  return (const void *)glfwGetProcAddress(symbol);
}

void swap_buffers(void *user) {
  glfwSwapBuffers((GLFWwindow *)user);
}

void make_current(void *user) {
  glfwMakeContextCurrent((GLFWwindow *)user);
}

void make_not_current(void *user) {
  glfwMakeContextCurrent(nullptr);
}

void window_size_callback(GLFWwindow* window, int width, int height) {
  EventHandler *event_handler = (EventHandler *)glfwGetWindowUserPointer(window);
  EventHandler_handle_resize(event_handler, DeviceSize { width, height });
}

void window_content_scale_callback(GLFWwindow* window, float xscale, float yscale) {
  EventHandler *event_handler = (EventHandler *)glfwGetWindowUserPointer(window);
  EventHandler_handle_scale_factor_change(event_handler, xscale);
}

void window_refresh_callback(GLFWwindow* window) {
  EventHandler *event_handler = (EventHandler *)glfwGetWindowUserPointer(window);
  EventHandler_handle_redraw(event_handler);
}

int main() {
  // std::cout << "foo" << std::endl;
  GLFWwindow* window;

  /* Initialize the library */
  if (!glfwInit()) return -1;

  /* Create a windowed mode window and its OpenGL context */
  window = glfwCreateWindow(640, 480, "Hello World", NULL, NULL);
  if (!window) {
    glfwTerminate();
    return -1;
  }

  glfwMakeContextCurrent(window);
  Gl *gl = load_gl(get_proc_address);

  int width, height;
  glfwGetWindowSize(window, &width, &height);

  float xscale, yscale;
  glfwGetWindowContentScale(window, &xscale, &yscale);

  Renderer *renderer = renderer_new(gl, xscale, DeviceSize { width, height });
  EventHandler *event_handler = EventHandler_new(renderer, swap_buffers, make_current, make_not_current, (void *)window);

  glfwSetWindowUserPointer(window, (void *)event_handler);

  glfwSetWindowSizeCallback(window, window_size_callback);
  glfwSetWindowContentScaleCallback(window, window_content_scale_callback);
  glfwSetWindowRefreshCallback(window, window_refresh_callback);

  while (!glfwWindowShouldClose(window)) {
    EventHandler_handle_empty(event_handler);
    /* Render here */
    // glClear(GL_COLOR_BUFFER_BIT);

    /* Swap front and back buffers */
    // glfwSwapBuffers(window);

    /* Poll for and process events */
    // glfwPollEvents();
    glfwWaitEvents();
  }

  EventHandler_drop(event_handler);

  glfwTerminate();
  return 0;
}