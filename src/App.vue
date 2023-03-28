<script setup lang="ts">
import WeekRow from "./components/WeekRow.vue";
import { ScheduleDate } from "./model/schedule-date";
import { InclusiveScheduleDateRange } from "./model/schedule-date-range";
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

const weeks = [
  Array.from(new InclusiveScheduleDateRange(
    ScheduleDate.fromGregorian({
    year: 2023,
    month: 2,
    day: 27
  }),
  ScheduleDate.fromGregorian({
    year: 2023,
    month: 3,
    day: 5
  })
  )),
  Array.from(new InclusiveScheduleDateRange(
    ScheduleDate.fromGregorian({
    year: 2023,
    month: 3,
    day: 6
  }),
  ScheduleDate.fromGregorian({
    year: 2023,
    month: 3,
    day: 12
  })
  )),
  Array.from(new InclusiveScheduleDateRange(
    ScheduleDate.fromGregorian({
    year: 2023,
    month: 3,
    day: 13
  }),
  ScheduleDate.fromGregorian({
    year: 2023,
    month: 3,
    day: 19
  })
  )),
]

</script>

<template>
  <div class="flex justify-center">
    <h1 class="mt-4 text-white text-2xl underline">Situps</h1>
  </div>
  <week-row :dates="weeks[0]" class="m-8"></week-row>
  <week-row :dates="weeks[1]" class="m-8"></week-row>
  <week-row :dates="weeks[2]" class="m-8"></week-row>
</template>
