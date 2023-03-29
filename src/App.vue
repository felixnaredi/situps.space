<script setup lang="ts">
import { json } from "body-parser";
import WeekRow from "./components/WeekRow.vue";
import { ScheduleDate } from "./model/schedule-date";
import { useEntriesStore } from "./store/entries-store";
import { Ref, ref } from "vue";

const weeks: Ref<Record<number, ScheduleDate[]>> = ref({});

useEntriesStore().setScheduleDateRange(
  ScheduleDate.fromGregorian({
    year: 2023,
    month: 3,
    day: 20
  }),
  ScheduleDate.fromGregorian({
    year: 2023,
    month: 4,
    day: 9
  })
)

weeks.value = useEntriesStore().weeks;

</script>

<template>
  <div class="flex justify-center">
    <h1 class="mt-4 text-white text-2xl underline">Situps</h1>
  </div>
  <week-row v-for="week in weeks" :key="week[0].week" :dates="week" class="m-8"></week-row>
</template>
