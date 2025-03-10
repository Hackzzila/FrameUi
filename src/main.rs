use std::sync::Arc;

use winit_adapter as window;

use window::glutin;

use project_a::dom;

fn main() {
  pretty_env_logger::init();

  // let f = std::fs::File::open("file.cframe").unwrap();
  // let doc = dom::CompiledDocument::load_from(f);

  // let doc = Arc::new(doc);

  let doc = dom::include_document!("../file.cframe");

  doc.scope.write().unwrap().push("id", "id".to_string());

  // let mut devtools = chrome_devtools::DevTools::new("127.0.0.1:4000");
  // devtools.add_view(Arc::clone(&doc));

  let event_loop = glutin::event_loop::EventLoop::with_user_event();

  // let notifier = Box::new(window::Notifier::new());
  let window = window::Window::new::<()>(
    glutin::window::WindowBuilder::new()
      .with_title("Foo")
      .with_inner_size(glutin::dpi::LogicalSize::new(1920, 1080)),
    &event_loop,
    event_loop.create_proxy(),
    Arc::clone(&doc),
  );

  let mut window = Some(window);

  event_loop.run(move |event, _, control_flow| {
    *control_flow = glutin::event_loop::ControlFlow::Wait;

    println!("{:?}", event);

    match &event {
      glutin::event::Event::WindowEvent { event, .. } => match event {
        glutin::event::WindowEvent::CloseRequested => {
          window.take().unwrap().deinit();

          *control_flow = glutin::event_loop::ControlFlow::Exit;

          return;
        }

        _ => {}
      },

      glutin::event::Event::LoopDestroyed => {
        return;
      }

      _ => {}
    }

    match &mut window {
      Some(window) => window.handle_event(&event),
      None => {}
    }
  });
}
