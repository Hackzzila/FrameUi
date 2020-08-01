<template>
  <div>
    <div class="bg-blue-500">
      <div class="text-center py-12">
        <div class="m-auto text-4xl font-inter font-medium">Documentation</div>
      </div>
    </div>

    <div class="container mx-auto mt-12 flex space-x-4 justify-around">
      <router-link to="layout" append class="bg-gray-300 p-6 rounded-lg text-center">
        <p class="text-2xl font-inter font-medium">Layout Docs</p>
      </router-link>

      <router-link to="css" append class="bg-gray-300 p-6 rounded-lg text-center">
        <p class="text-2xl font-inter font-medium">CSS Property Docs</p>
      </router-link>

      <router-link to="sass" append class="bg-gray-300 p-6 rounded-lg text-center">
        <p class="text-2xl font-inter font-medium">Sass Docs</p>
      </router-link>

      <router-link to="high-level" append class="bg-gray-300 p-6 rounded-lg text-center">
        <p class="text-2xl font-inter font-medium">High-level Embedding Docs</p>
      </router-link>

      <router-link to="low-level" append class="bg-gray-300 p-6 rounded-lg text-center">
        <p class="text-2xl font-inter font-medium">Low-level Embedding Docs</p>
      </router-link>
    </div>
  </div>
</template>
