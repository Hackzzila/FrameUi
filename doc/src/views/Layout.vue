<template>
  <div>
    <div class="bg-blue-500">
      <div class="text-center py-12">
        <div class="m-auto text-4xl font-inter font-medium">&lt;Button&gt;</div>
      </div>
    </div>

    <div class="flex mt-12 container mx-auto">
      <div class="pl-4 pr-12">
        <p class="text-lg font-medium">Elements</p>
        <a class="block" href="google.com">&lt;Frame&gt;</a>
        <a class="block" href="google.com">&lt;Head&gt;</a>
        <a class="block" href="google.com">&lt;Body&gt;</a>
        <a class="block text-blue-500" href="google.com">&lt;Button&gt;</a>
      </div>

      <div class="flex-grow flex flex-col">
        <p class="text-2xl font-medium">Overview</p>
        <pre class="bg-gray-200 text-lg rounded-lg px-4 py-3"><code>&lt;Button class="string" id="string" style="string" disabled="bool"&gt;...&lt;Button/&gt;</code></pre>

        <div class="ml-1 pl-2 mt-4 border-l-2 border-gray-300">
          <table class="table-auto flex-grow">
            <tbody>
              <tr>
                <td class="font-medium whitespace-no-wrap">Name</td>
                <td class="pl-4 w-full">Button</td>
              </tr>
              <tr>
                <td class="font-medium whitespace-no-wrap">Valid Inside</td>
                <td class="pl-4">Body Context</td>
              </tr>
              <tr>
                <td class="font-medium whitespace-no-wrap">Children</td>
                <td class="pl-4">Allowed</td>
              </tr>
              <tr>
                <td class="font-medium whitespace-no-wrap">Child Context</td>
                <td class="pl-4">Body Context</td>
              </tr>
              <tr>
                <td class="font-medium whitespace-no-wrap">Since</td>
                <td class="pl-4">1.45.0</td>
              </tr>
            </tbody>
          </table>
        </div>

        <p class="text-2xl font-medium mt-8">Attributes</p>
        <table class="table-auto flex-grow">
          <thead>
            <tr class="text-left">
              <th class="px-4 py-2">Attribute</th>
              <th class="px-4 py-2">Type</th>
              <th class="px-4 py-2">Default</th>
              <th class="px-4 py-2 w-full">Description</th>
              <th class="px-4 py-2">Since</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td class="border px-4 py-2">class</td>
              <td class="border px-4 py-2">string</td>
              <td class="border px-4 py-2 text-sm"><code>""</code></td>
              <td class="border px-4 py-2">A space-separated list of the classes of the element</td>
              <td class="border px-4 py-2">1.0.0</td>
            </tr>
            <tr class="bg-gray-100">
              <td class="border px-4 py-2">id</td>
              <td class="border px-4 py-2">string</td>
              <td class="border px-4 py-2 text-sm"><code>""</code></td>
              <td class="border px-4 py-2">Defines a unique identifier (ID) which must be unique in the whole document</td>
              <td class="border px-4 py-2">1.1.0</td>
            </tr>
            <tr>
              <td class="border px-4 py-2">style</td>
              <td class="border px-4 py-2">string</td>
              <td class="border px-4 py-2 text-sm"><code>""</code></td>
              <td class="border px-4 py-2">A space-separated list of the classes of the element</td>
              <td class="border px-4 py-2">1.23.0</td>
            </tr>
            <tr class="bg-gray-100">
              <td class="border px-4 py-2">disabled</td>
              <td class="border px-4 py-2">bool</td>
              <td class="border px-4 py-2 text-sm"><code>false</code></td>
              <td class="border px-4 py-2">Whether the button is disabled</td>
              <td class="border px-4 py-2">1.46.0</td>
            </tr>
          </tbody>
        </table>

        <p class="text-2xl font-medium mt-8">Default Style</p>
        <pre class="bg-gray-200 rounded-lg px-4 py-3"><code>Button {
  width: 2px;
  width: 2px;
  width: 2px;
  width: 2px;
  width: 2px;
  width: 2px;
  width: 2px;
  width: 2px;
  width: 2px;
  width: 2px;
  width: 2px;
  width: 2px;
}</code></pre>

      </div>
    </div>
  </div>
</template>
