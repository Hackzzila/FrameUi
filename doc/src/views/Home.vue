<template>
  <div>
    <div class="bg-blue-500">
      <div class="text-center py-32">
        <div class="m-auto text-4xl font-inter font-medium">The cross-platform, cross-language UI Framework</div>
      </div>
    </div>

    <div class="container mx-auto mt-12 grid grid-cols-2">
      <div class="m-auto text-center">
        <p class="text-3xl font-inter font-medium">Write HTML-like layouts</p>
        <p>Using a syntax familiar to HTML</p>
      </div>

      <pre class="bg-gray-200 rounded-lg px-4 py-3">
<code>&lt;Page&gt;
  &lt;Meta&gt;

  &lt;/Meta&gt;

  &lt;Body&gt;
    foo
    &lt;!-- bar --&gt;
    &lt;![CDATA[characters with markup]]&gt;
    baz
    &lt;!-- &lt;foo:Bar/&gt; --&gt;
    &lt;?pi?&gt;
    &lt;?pi2 foo bar="baz"?&gt;
  &lt;/Body&gt;
&lt;/Page&gt;</code></pre>
    </div>

    <div class="container grid grid-cols-2 bg-gray-900 p-12 rounded-lg mx-auto mt-12 text-gray-100">
      <pre class="bg-gray-800 rounded-lg px-4 py-3">
<code>.some-class {
  width: 2px;
  width: 2px;
  width: 2px;
  width: 2px;
}

.some-class {
  width: 2px;
  width: 2px;
  width: 2px;
  width: 2px;
}

.some-class {
  width: 2px;
  width: 2px;
  width: 2px;
  width: 2px;
}</code></pre>

      <div class="m-auto text-center">
        <p class="text-3xl font-inter font-medium">Style with CSS or Sass</p>
        <p>With CSS selectors and familiar rules</p>
      </div>
    </div>

    <div class="container mx-auto mt-12 grid grid-cols-2">
      <div class="m-auto text-center p-6">
        <p class="text-3xl font-inter font-medium">Run anywhere</p>
        <p>FrameUi can be used in most languages and platforms</p>
      </div>

      <pre class="bg-gray-200 rounded-lg px-4 py-3">
<code>let event_loop = glutin::event_loop::EventLoop::new();
let proxy = event_loop.create_proxy();
let notifier = Box::new(window::Notifier::new(proxy));
let window = window::Window::new(
  glutin::window::WindowBuilder::new()
    .with_title("Foo")
    .with_inner_size(glutin::dpi::LogicalSize::new(1920, 1080)),
  &event_loop,
  notifier,
);</code></pre>
    </div>

    <div class="container grid grid-cols-2 bg-gray-900 p-12 rounded-lg mx-auto mt-12 text-gray-100">
      <div class="mx-auto text-center">
        <p class="text-3xl font-inter font-medium">Language Support</p>
        <p>FrameUi can be used in mosy programming languages</p>

        <p class="text-2xl font-inter font-medium mt-4">Tier 1</p>
        <p class="-mt-2 mb-4 text-sm">Tier 1 languages support every feature and module</p>
        <p>Rust</p>

        <p class="text-2xl font-inter font-medium mt-4">Tier 2</p>
        <p class="-mt-2 mb-4 text-sm">Tier 2 languages support most features and modules</p>
        <p>C</p>
        <p>C++</p>
        <p>Swift</p>

        <p class="text-2xl font-inter font-medium mt-4">Tier 3</p>
        <p class="-mt-2 mb-4 text-sm">Tier 3 languages support only high-level modules</p>
        <p>Python</p>
        <p>C#</p>
        <p>Node.js</p>
        <p>Java</p>
        <p>Go</p>
      </div>

      <div class="mx-auto text-center">
        <p class="text-3xl font-inter font-medium">Platform Support</p>
        <p>FrameUi runs on every platform with <a href="https://forge.rust-lang.org/release/platform-support.html" class="underline">Rust stdlib support</a> and OpenGL or OpenGL ES</p>
        <br>
        <br>
        <p>Windows</p>
        <p>macOS</p>
        <p>Linux</p>
        <p>Android</p>
        <p>iOS</p>
        <p>...</p>
      </div>
    </div>

    <div class="container mx-auto mt-12 grid grid-cols-4">
      <div class="bg-gray-300 p-6 rounded-lg text-center">
        <p class="text-2xl font-inter font-medium">HTML-inspired layout</p>
        <p>For most developers</p>
      </div>

      <div class="bg-gray-300 p-6 rounded-lg text-center">
        <p class="text-2xl font-inter font-medium">Standard CSS Syntax</p>
        <p>For most developers</p>
      </div>

      <div class="bg-gray-300 p-6 rounded-lg text-center">
        <p class="text-2xl font-inter font-medium">Native Sass Support</p>
        <p>For most developers</p>
      </div>

      <div class="bg-gray-300 p-6 rounded-lg text-center">
        <p class="text-2xl font-inter font-medium">Flexbox implementation</p>
        <p>For most developers</p>
      </div>

      <div class="bg-gray-300 p-6 rounded-lg text-center">
        <p class="text-2xl font-inter font-medium">Layouts compile to binary for production</p>
        <p>For most developers</p>
      </div>
    </div>
  </div>
</template>
