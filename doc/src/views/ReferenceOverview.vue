<template>
  <div>
    <div v-for="mod in languages[current].modules" v-bind:key="mod.name" class="mb-6">
      <h1 class="text-xl font-inter">{{mod.name}}</h1>
      <div class="ml-1 pl-2 border-l-2 border-gray-300">
        <div v-for="(child, idx) in mod.children" v-bind:key="idx">
          <router-link :to="`${mod.name}/${child.name}`" append class="text-blue-500">
            <code class="font-inter">
              <span class="text-gray-600">{{languages[current].keywords[child.type]}}</span>
              {{child.name}}
            </code>
            <br>
          </router-link>
          <div v-if="child.brief" class="ml-2">{{child.brief}}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: 'DocOverview',
  props: ['current', 'languages'],
};
</script>
