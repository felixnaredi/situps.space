<script setup lang="ts">
import Cell from "./Cell.vue";
import { ScheduleDate } from "../scheme/schedule-date";

defineProps<{ scheduleDates: ScheduleDate[] }>()

</script>

<template>
    <div class="p-5 flex flex-wrap justify-start">
        <cell v-for="date in scheduleDates" :key="`${date.year}${date.month}${date.day}`"
            class="m-2 border-4 border-solid border-stone-800" :schedule-date="date">
        </cell>
    </div>
</template>
