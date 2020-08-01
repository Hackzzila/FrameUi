<template>
  <div>
    <!-- <div class="container mx-auto py-4 flex justify-between items-center">
      <router-link to="/" class="text-3xl font-logo">Frame<span class="text-blue-500">Ui</span></router-link>

      <div class=" flex space-x-4">
        <router-link class="hover:text-blue-500" active-class="text-blue-500" to="/docs">Docs</router-link>
        <a class="hover:text-blue-500" href="https://github.com/Hackzzila/FrameUi">GitHub</a>
      </div>
    </div> -->

    <div class="bg-gray-800 text-white py-2">
      <div class="container mx-auto flex justify-between items-center">
        <p class="text-2xl text-gray-100" style="font-family: 'Montserrat', sans-serif;">Frame<span class="text-blue-500">Ui</span> <span class="font-thin"></span></p>

        <!-- <select class="bg-transparent border border-gray-600 rounded-md w-20 my-1" v-model="current">
          <option class="text-black" v-for="(language, idx) in languages" v-bind:key="idx" :value="idx">{{language.language}}</option>
        </select> -->
        <div class="flex space-x-4">
          <router-link class="hover:text-blue-500" active-class="text-blue-500" to="/docs">Docs</router-link>
          <a class="hover:text-blue-500" href="https://github.com/Hackzzila/FrameUi">GitHub</a>
        </div>
      </div>
    </div>

    <router-view/>

    <div class="bg-gray-900 text-white mt-12">
      <div class="container mx-auto py-4">
        <p class="text-3xl font-logo text-gray-700">Frame<span class="text-blue-700">Ui</span></p>

        <div class="underline font-light">
          <router-link active-class="text-blue-500" to="/docs">Docs</router-link>
        </div>

        <p class="text-xs">Copyright &copy; 2020 Zachary Vacura</p>
      </div>
    </div>
  </div>
</template>
