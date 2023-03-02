import { ScheduleDate } from "./schedule-date";

/**
 * Key used to identify an entry on the server and frontend.
 */
export interface EntryKey {
  readonly userID: string;
  readonly scheduleDate: ScheduleDate;
}

export function EntryKeyIdentifier(entryKey: EntryKey): string {
  const userID = entryKey.userID;
  const year = entryKey.scheduleDate.year;
  const month = entryKey.scheduleDate.month;
  const day = entryKey.scheduleDate.day;
  return JSON.stringify({ userID, scheduleDate: { year, month, day } });
}

export interface EntryData {
  readonly amount: number;
}
