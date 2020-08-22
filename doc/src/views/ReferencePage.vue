<template>
  <div>
    <h1 class="text-3xl font-inter"><span class="text-gray-600">{{languages[current].keywords[data.type]}}</span> {{data.name}}</h1>
    <div class="ml-1 pl-2 border-l-2 border-gray-300">
      {{data.brief}}
    </div>

    <StructView :languages="languages" :current="current" :data="data" v-if="data.type == 'Struct'"/>
    <DataStructView :data="data" v-if="data.type == 'DataStruct'"/>
    <TypedefView :data="data" v-if="data.type == 'Typedef'"/>
  </div>
</template>

<script>
// @ is an alias to /src
import StructView from '@/components/StructView.vue';
import DataStructView from '@/components/DataStructView.vue';
import TypedefView from '@/components/TypedefView.vue';

export default {
  name: 'DocPage',
  props: ['current', 'languages', 'mod', 'item'],
  components: {
    StructView,
    DataStructView,
    TypedefView,
  },
  data() {
    return {
      data: this.$props.languages[this.$props.current].modules.find((x) => x.name === this.$props.mod).children.find((x) => x.name === this.$props.item),
    };
  },
};
</script>
