<template>
  <div>
    <div class="mt-2 ml-1 pl-2 border-l-2 border-gray-300">
      <div v-for="member in data.methods" v-bind:key="member.name">
        <a :href="`#${member.name}`" class="text-blue-500"><code>{{member.name}}</code><br></a>
        <div v-if="member.brief" class="ml-2">{{member.brief}}</div>
      </div>
    </div>

    <pre class="font-sans">{{data.description}}</pre>

    <hr class="border-gray-300 border-t-2 my-8">

    <div v-for="member in data.methods" v-bind:key="member.name" :id="member.name" class="mb-12">
      <pre class="bg-gray-200 rounded-lg px-4 py-3 mb-4"><code>{{member.declaration}}</code></pre>
      <div class="ml-1 pl-2 border-l-2 border-gray-300">
        <pre class="font-sans">{{member.description}}</pre>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: 'StructView',
  props: {
    data: Object,
  },
};
</script>
