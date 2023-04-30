<script setup lang="ts">
import { User } from "../interface/user";
import { ScheduleDate } from "../model/schedule-date";
import { Ref, ref, computed } from "vue";
import { useEntriesStore } from "../store/entries-store";

const props = defineProps<{ user: User, scheduleDate: ScheduleDate }>();

const amount: Ref<null | number> = ref(null);

const entryKey = computed(() => {
  return {
    userId: props.user._id, scheduleDate: {
      year: props.scheduleDate.year,
      month: props.scheduleDate.month,
      day: props.scheduleDate.day,
    }
  };
});

//
// Fetch data for the entry.
//
// TODO:
//   Modifying any parameter of `entryKey` should call this again.
//
useEntriesStore().getEntry(entryKey.value, (response) => {
  if (response.entryData != null) {
    amount.value = response.entryData.amount;
  }
});

//
// Subscribe to changes of the entry.
//
// TODO:
//   Modifying any parameter of `entryKey` should remove the callback from the broadcaster and add
//   a callback for the new `entryKey`.
//
useEntriesStore().subscribeToStateChange(entryKey.value, (message) => {
  if (message.newValue.amount) {
    amount.value = message.newValue.amount;
  } else {
    amount.value = null;
  }
});

async function updateAmount(event: Event) {
  const value = (event.target! as HTMLInputElement).value;
  if (value == "") {
    useEntriesStore().updateEntry(entryKey.value, null);
  } else {
    useEntriesStore().updateEntry(entryKey.value, Number(value));
  }
}


</script>

<template>
  <!-- 
        TODO:
          Theme styles should be resolved in another file.
      -->
  <div class="border-t-2 border-solid border-stone-700" :class="{
    'bg-sky-200': user.theme == 'ocean' && amount == null,
    'bg-sky-500': user.theme == 'ocean' && amount != null,
    'bg-green-200': user.theme == 'forrest' && amount == null,
    'bg-green-500': user.theme == 'forrest' && amount != null,
  }">
    <h3 class="ml-2">{{ user.displayName }}</h3>
    <div>
      <input type="text" class="w-full p-1" v-model="amount" @blur="updateAmount"
        @keyup.enter="event => (event.target! as HTMLInputElement).blur()" />
    </div>
  </div>
</template>