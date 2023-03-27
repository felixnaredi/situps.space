<script setup lang="ts">
import WeekRow from "./components/WeekRow.vue";
import { ScheduleDate } from "./model/schedule-date";
import { useEntriesStore } from "./store/entries-store";

useEntriesStore().setScheduleDateRange(
  ScheduleDate.fromGregorian({
    year: 2023,
    month: 2,
    day: 27
  }),
  ScheduleDate.fromGregorian({
    year: 2023,
    month: 3,
    day: 26
  })
)

</script>

<template>
  <div class="flex justify-center">
    <h1 class="mt-2 text-white text-2xl underline">Situps</h1>
  </div>
  <week-row class="m-8"></week-row>
  <week-row class="m-8"></week-row>
  <week-row class="m-8"></week-row>
</template>

<style scoped></style>
