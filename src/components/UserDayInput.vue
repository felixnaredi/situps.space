<script setup lang="ts">
import { User } from "../interface/user";
import { ScheduleDate } from "../interface/schedule-date";
import { Ref, ref, computed } from "vue";
import { useEntriesStore } from "../store/entries-store";

const props = defineProps<{ user: User, scheduleDate: ScheduleDate }>();

const amount: Ref<null | number> = ref(null);

const entryKey = computed(() => {
  return { userID: props.user.userID, scheduleDate: props.scheduleDate };
});

//
// Fetch data for the entry.
//
// TODO:
//   Modifying any parameter of `entryKey` should call this again.
//
useEntriesStore().getEntry(entryKey.value, (response) => {
  if (response.amount != null) {
    amount.value = response.amount;
  }
});

async function updateAmount(event: Event) {
  const amount = Number((event.target! as HTMLInputElement).value);

  fetch("/api/entry/update-amount", {
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