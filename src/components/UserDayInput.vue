<script setup lang="ts">
import { User } from "../scheme/user";
import { ScheduleDate } from "../scheme/schedule-date";
import { ref } from "vue";

const props = defineProps<{ user: User, scheduleDate: ScheduleDate }>();
const amount = ref(null);

fetch("/api/amount", {
  headers: {
    "Content-Type": "application/json",
  },
  method: "POST",
  body: JSON.stringify({
    userID: props.user.userID,
    scheduleDate: props.scheduleDate,
  })
}).then(async response => {
  const obj = await response.json();
  if (obj.amount != null) {
    amount.value = obj.amount;
  }
});

async function updateAmount(event: Event) {
  const amount = Number((event.target! as HTMLInputElement).value);

  fetch("/api/update", {
    headers: {
      "Content-Type": "application/json"
    },
    method: "POST",
    body: JSON.stringify({
      userID: props.user.userID,
      scheduleDate: props.scheduleDate,
      amount
    })
  }).then(console.log);
}

</script>

<template>
  <div class="border-t-2 border-solid border-stone-700" :class="{
    'bg-sky-200': user.theme == 'ocean' && amount == null,
    'bg-sky-500': user.theme == 'ocean' && amount != null,
    'bg-green-200': user.theme == 'forest' && amount == null,
    'bg-green-500': user.theme == 'forest' && amount != null,
  }">
    <h3 class="ml-2">{{ user.displayName }}</h3>
    <div>
      <input type="text" class="w-full p-1" v-model="amount" @blur="updateAmount"
        @keyup.enter="event => (event.target! as HTMLInputElement).blur()" />
    </div>
  </div>
</template>