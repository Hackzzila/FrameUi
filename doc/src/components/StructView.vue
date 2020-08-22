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
      <CodeBlock :current="current" :languages="languages" :item="data.name" :text="member.declaration" />
      <div class="ml-1 pl-2 border-l-2 border-gray-300">
        <pre class="font-sans">{{member.description}}</pre>
      </div>
    </div>
  </div>
</template>

<script>
import CodeBlock from './CodeBlock.vue';

export default {
  name: 'StructView',
  props: ['current', 'languages', 'data'],
  components: { CodeBlock },
};
</script>
