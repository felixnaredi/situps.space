import { defineStore } from "pinia";
import { ScheduleDate } from "../interface/schedule-date";
import { Ref, ref, computed } from "vue";
import { InclusiveScheduleDateRange } from "../model/schedule-date-range";
import { io } from "socket.io-client";
import {
  EntryEventGetResponse,
  EntryEventStateChange,
} from "../interface/response";
import { EntryKey, EntryKeyIdentifier } from "../interface/entry";

/**
 * Store used to keep track of the state of the entries.
 */
export const useEntriesStore = defineStore("entries", () => {
  //
  // Create socket under the namespace "/entry".
  //
  const socket = io(`${import.meta.env.SITUPS_WS_URL}/entry`);

  //
  // Respond to established connection.
  //
  socket.on("connect", () => {
    console.log("`useEntriesStore` connected to web-socket");
    socket.emit("ack-connect");
  });

  const scheduleDatesRef: Ref<ScheduleDate[]> = ref([]);

  const scheduleDates = computed(() => scheduleDatesRef.value);

  function setScheduleDateRange(from: ScheduleDate, to: ScheduleDate) {
    scheduleDatesRef.value = Array.from(
      new InclusiveScheduleDateRange(from, to)
    );
  }

  function getEntry(
    entryKey: EntryKey,
    callback: (response: EntryEventGetResponse) => void
  ) {
    socket.emit("get", { entryKey }, callback);
  }

  function updateEntry(entryKey: EntryKey, amount: null | number) {
    socket.emit("update", { entryKey, newValue: { amount } });
  }

  // TODO:
  //   This broadcaster is very simple. Make it more robust or find a third party library for it.

  const stateChangeListeners: Record<
    string,
    (message: EntryEventStateChange) => void
  > = {};

  function subscribeToStateChange(
    key: EntryKey,
    callback: (message: EntryEventStateChange) => void
  ) {
    stateChangeListeners[EntryKeyIdentifier(key)] = callback;
  }

  socket.on("state-changed", (message: EntryEventStateChange) => {
    console.log("state-change", message);

    const callback = stateChangeListeners[EntryKeyIdentifier(message.entryKey)];
    if (callback != undefined) {
      callback(message);
    }
  });

  return {
    /**
     * Sets the range of scheduled dates.
     */
    setScheduleDateRange,

    /**
     * Updates the state of a entry.
     *
     * @param entryKey Key to the entry to update.
     * @param amount New value.
     */
    updateEntry,

    /**
     * Subscribes to the websocket event 'state-change' and calls `callback` every time `entryKey` of
     * the message is equal to `key`.
     *
     * @param key Key for the entry that the broadcaster will notify about.
     * @param callback Function to be called.
     */
    subscribeToStateChange,

    /**
     * Gets the data of the entry with identified by `entryKey`.
     *
     * @param entryKey Entry identification.
     * @param callback Handler for the `EntryGetResponse`.
     */
    getEntry,

    /**
     * The scheduled dates of the entries.
     */
    scheduleDates,
  };
});
