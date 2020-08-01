<template>
  <div>
    <div class="bg-gray-800 text-white py-2">
      <div class="container mx-auto flex justify-between">
        <p class="text-2xl text-gray-100" style="font-family: 'Montserrat', sans-serif;">Frame<span class="text-blue-500">Ui</span> <span class="font-thin">Docs</span></p>

        <select class="bg-transparent border border-gray-600 rounded-md w-20 my-1" v-model="current">
          <option class="text-black" v-for="(language, idx) in languages" v-bind:key="idx" :value="idx">{{language.language}}</option>
        </select>
      </div>
    </div>
    <div class="container mx-auto mt-6">
      <router-view :current="current" :languages="languages" />
    </div>
  </div>
</template>

<script>
const c = require('../doc.json');

export default {
  name: 'Doc',
  data() {
    return {
      current: 'c',
      languages: { c },
    };
  },
};
</script>
