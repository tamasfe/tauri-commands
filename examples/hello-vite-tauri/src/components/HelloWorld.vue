<template>
  <div>
    <h1>Deferring heavy computation to Rust</h1>
    <label for="a"></label>
    <input id="a" type="number" v-model="a" />
    +
    <label for="b"></label>
    <input id="b" type="number" v-model="b" />
    = {{ result }}
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { addNumbers } from "../tauri";

const a = ref(1);
const b = ref(1);

const result = ref<number | undefined>(undefined);

watch(
  () => [a.value, b.value],
  async ([a, b]) => {
    result.value = await addNumbers(a, b);
  },
  { immediate: true }
);
</script>
