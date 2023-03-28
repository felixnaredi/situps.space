<script setup lang="ts">

import { ScheduleDate } from '../model/schedule-date';
import DayCard from "./DayCard.vue";
import { Ref, ref, onMounted } from "vue";

defineProps<{ dates: ScheduleDate[] }>();

const separatorWidth = ref(0);
const dayCardContainer: Ref<null | HTMLElement> = ref(null);

const resizeObserver = new ResizeObserver(() => {
    //
    // Set the width of the border that separates `WeekRow`s based on the position of the right
    // corner of the rightmost card.
    //
    if (dayCardContainer.value != null) {
        let x = 0;
        for (const element of dayCardContainer.value.children) {
            const child = element as HTMLElement;
            x = Math.max(child.offsetLeft + child.clientWidth, x);
        };
        separatorWidth.value = x - dayCardContainer.value.offsetLeft;
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
        <h2 class="text-slate-200 text-xl mb-4">{{ dates[0].week }}</h2>
        <div ref="dayCardContainer" class="flex flex-wrap justify-stretch">
            <!-- TODO: -->
            <!-- Using `dayOffset` as key can repeat if dates from different years are presented. -->
            <day-card v-for="date in dates" :key="date.dayOffset" :date="date" class="mb-6 mr-8" />
        </div>
    </div>
</template>

