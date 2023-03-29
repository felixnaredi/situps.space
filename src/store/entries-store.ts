import { defineStore } from "pinia";
import { ScheduleDate, Weekday } from "../model/schedule-date";
import { Ref, ref, computed } from "vue";
import { InclusiveScheduleDateRange } from "../model/schedule-date-range";
import {
  EntryEventGetResponse,
  EntryEventStateChange,
} from "../interface/response";
import { EntryKey, EntryKeyIdentifier } from "../interface/entry";

/**
 * Store used to keep track of the state of the entries.
 */
export const useEntriesStore = defineStore("entries", () => {
  const getEntryDataListeners: Record<
    string,
    undefined | ((event: EntryEventGetResponse) => void)
  > = {};

  const stateChangeListeners: Record<
    string,
    (message: EntryEventStateChange) => void
  > = {};

  const socket = connect();
  socket.then((socket) => {
    socket.addEventListener("message", (message) => {
      const response = JSON.parse(message.data);

      console.log(response);

      if (response.getEntryData) {
        const r: EntryEventGetResponse = response.getEntryData;
        const key = EntryKeyIdentifier(r.entryKey);

        const callback = getEntryDataListeners[key];
        if (callback) {
          getEntryDataListeners[key] = undefined;
          callback(r);
        }
      } else if (response.updateEntry) {
        const r: EntryEventStateChange = {
          entryKey: response.updateEntry._id,
          oldValue: null,
          newValue: response.updateEntry.value,
        };
        stateChangeListeners[EntryKeyIdentifier(r.entryKey)](r);
      }
    });
  });

  const scheduleDatesRef: Ref<ScheduleDate[]> = ref([]);
  const scheduleDates = computed(() => scheduleDatesRef.value);

  const weeks = computed(() => {
    return scheduleDatesRef.value.reduce(
      (acc: Record<number, ScheduleDate[]>, date) => {
        (acc[date.week] = acc[date.week] || []).push(date);
        return acc;
      },
      {}
    );
  });

  // TODO:
  //   The behavior of the range is a bit quirky and should probably be more robustly defined.
  function setScheduleDateRange(from: ScheduleDate, to: ScheduleDate) {
    from = from.reversedToWeekday(Weekday.MONDAY);
    to = to.reversedToWeekday(Weekday.SUNDAY);
    scheduleDatesRef.value = Array.from(
      new InclusiveScheduleDateRange(from, to)
    );
  }

  async function getEntry(
    entryKey: EntryKey,
    callback: (response: EntryEventGetResponse) => void
  ) {
    getEntryDataListeners[EntryKeyIdentifier(entryKey)] = callback;

    const s = await socket;
    s.send(
      JSON.stringify({
        getEntryData: { entryKey: entryKey },
      })
    );
  }

  async function updateEntry(entryKey: EntryKey, amount: null | number) {
    const s = await socket;
    s.send(
      JSON.stringify({
        updateEntry: {
          entry: {
            _id: {
              userId: entryKey.userId,
              scheduleDate: entryKey.scheduleDate,
            },
            value: {
              amount: amount,
            },
          },
        },
      })
    );
  }

  function subscribeToStateChange(
    key: EntryKey,
    callback: (message: EntryEventStateChange) => void
  ) {
    stateChangeListeners[EntryKeyIdentifier(key)] = callback;
  }

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

    /**
     * The scheduled dates grouped by week number.
     */
    weeks,
  };
});

function connect(): Promise<WebSocket> {
  //
  // Create socket under the namespace "/entry".
  //
  return new Promise((resolve, reject) => {
    const socket = new WebSocket(`${import.meta.env.SITUPS_V2_WS_URL}/entry`);
    socket.addEventListener("open", (event) => {
      console.log(event);
      resolve(socket);
    });
  });
}
