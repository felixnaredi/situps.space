<script setup lang="ts">

import { ScheduleDate } from '../model/schedule-date';
import DayCard from "./DayCard.vue";
import { Ref, ref, onMounted, ComponentPublicInstance } from "vue";

const date = ScheduleDate.fromGregorian({
    day: 1,
    month: 1,
    year: 2021,
});

const separatorWidth = ref(0);
const dayCardContainer: Ref<null | HTMLElement> = ref(null);

const resizeObserver = new ResizeObserver(() => {
    if (dayCardContainer.value != null) {
        let maxX = 0;
        dayCardContainer.value.childNodes.forEach((node) => {
            const child = node as HTMLElement;
            maxX = Math.max(child.offsetLeft + child.clientWidth, maxX);
        });
        separatorWidth.value = maxX - dayCardContainer.value.offsetLeft;
    }
});

onMounted(() => {
    if (dayCardContainer.value != null) {
        resizeObserver.observe(dayCardContainer.value);
    }
});
</script>

<template>
    <div>
        <div class="pt-4 border-t-solid border-t-2 border-t-orange-400" :style="{ width: separatorWidth + 'px' }"></div>
        <h2 class="text-slate-200 text-xl mb-4">{{ date.week }}</h2>
        <div class="flex flex-wrap justify-stretch" ref="dayCardContainer">
            <day-card :scheduleDate="date" class="mb-6 mr-8"></day-card>
            <day-card :scheduleDate="date" class="mb-6 mr-8"></day-card>
            <day-card :scheduleDate="date" class="mb-6 mr-8"></day-card>
            <day-card :scheduleDate="date" class="mb-6 mr-8"></day-card>
            <day-card :scheduleDate="date" class="mb-6 mr-8"></day-card>
            <day-card :scheduleDate="date" class="mb-6 mr-8"></day-card>
            <day-card :scheduleDate="date" class="mb-6 mr-8"></day-card>
        </div>
    </div>
</template>

