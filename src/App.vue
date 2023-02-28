<script setup lang="ts">
import Grid from "./components/Grid.vue";

const scheduleDates = [
  {
    year: 2023,
    month: 2,
    day: 27
  },
  {
    year: 2023,
    month: 2,
    day: 28
  },
  {
    year: 2023,
    month: 3,
    day: 1
  },
  {
    year: 2023,
    month: 3,
    day: 2
  },
  {
    year: 2023,
    month: 3,
    day: 3
  },
  {
    year: 2023,
    month: 3,
    day: 4
  },
  {
    year: 2023,
    month: 3,
    day: 5
  },
]

</script>

<template>
  <div class="flex justify-center">
    <h1 class="mt-2 text-white text-2xl underline">Situps</h1>
  </div>
  <grid :schedule-dates="scheduleDates"></grid>
</template>

<style scoped>
</style>
