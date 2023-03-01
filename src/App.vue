<script setup lang="ts">
import Grid from "./components/Grid.vue";
import { useEntriesStore } from "./store/entries-store";

useEntriesStore().setScheduleDateRange(
  {
    year: 2023,
    month: 2,
    day: 27
  },
  {
    year: 2023,
    month: 3,
    day: 5
  }
)

</script>

<template>
  <div class="flex justify-center">
    <h1 class="mt-2 text-white text-2xl underline">Situps</h1>
  </div>
  <grid></grid>
</template>

<style scoped></style>
