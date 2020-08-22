<template>
  <div>
    <div class="bg-gray-900 text-white py-2">
      <div class="container mx-auto flex justify-between">
        <p class="text-lg text-gray-100">Documentation</p>

        <div class="flex">
          <div class="mr-4">
            <span class="mr-1 text-sm text-gray-500">Version:</span>
            <select class="bg-transparent text-sm pr-1" v-model="current">
              <option class="text-black" v-for="(language, idx) in languages" v-bind:key="idx" :value="idx">master</option>
            </select>
          </div>

          <div>
            <span class="mr-1 text-sm text-gray-500">Language:</span>
            <select class="bg-transparent text-sm pr-1" v-model="current">
              <option class="text-black" v-for="(language, idx) in languages" v-bind:key="idx" :value="idx">{{language.language}}</option>
            </select>
          </div>
        </div>
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
