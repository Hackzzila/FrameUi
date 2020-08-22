<template>
  <pre class="bg-gray-200 rounded-lg px-4 py-3 mb-4"><code v-html="html"></code></pre>
</template>

<script>
export default {
  name: 'CodeBlock',
  props: ['current', 'languages', 'item', 'text'],
  computed: {
    html() {
      let code = this.$props.text;

      for (const module of this.$props.languages[this.$props.current].modules) {
        for (const item of module.children) {
          if (item.name !== this.$props.item) {
            code = code.replace(new RegExp(item.name, 'g'), `<router-link to="../../${module.name}/${item.name}" class="text-blue-500">${item.name}</router-link>`);
          }
        }
      }

      return code;
    },
  },
};
</script>
