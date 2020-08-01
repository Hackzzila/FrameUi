<template>
  <div class="mt-6">
    <pre class="bg-gray-200 rounded-lg px-4 py-3 mb-4"><code>{{data.declaration}}</code></pre>
    <div class="ml-1 pl-2 border-l-2 border-gray-300">
      <pre class="font-sans">{{data.description}}</pre>
    </div>
  </div>
</template>

<script>
export default {
  name: 'TypedefView',
  props: {
    data: Object,
  },
};
</script>
