<script setup lang="ts">
import { Ref, ref } from "vue";
import { User } from "../interface/user";
import UserDayInput from "./UserDayInput.vue";
import { ScheduleDate } from "../model/schedule-date"

defineProps<{ date: ScheduleDate }>();

let users: Ref<User[]> = ref([])

fetch("/api/users").then(response => response.json().then(obj => users.value = obj));

function displayScheduleDate(date: ScheduleDate) {
  function pad(x: number): string {
    return ("00" + x).slice(-2);
  }
  return `${date.year}-${pad(date.month)}-${pad(date.day)}`;
}

</script>

<template>
  <div class="w-40">
    <div class="flex justify-center bg-slate-700">
      <h2 class="text-gray-200">{{ displayScheduleDate(date) }}</h2>
    </div>
    <user-day-input v-for="user in users" :key="user._id" :user="user" :schedule-date="date">
    </user-day-input>
  </div>
</template>

