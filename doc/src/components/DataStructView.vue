<template>
  <div>
    <div class="mt-2 ml-1 pl-2 border-l-2 border-gray-300">
      <div v-for="member in data.fields" v-bind:key="member.name">
        <div>
          <code>
            <span class="text-gray-600">{{member.declaration.split(' ')[0]}}</span> <span>{{member.declaration.split(' ')[1]}}</span>
          </code>
          <br>
        </div>
        <div v-if="member.description" class="ml-2">{{member.description}}</div>
      </div>
    </div>

    <pre class="font-sans">{{data.description}}</pre>
  </div>
</template>

<script>
export default {
  name: 'DataStructView',
  props: {
    data: Object,
  },
};
</script>
