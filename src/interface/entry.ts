import { ScheduleDate } from "./schedule-date";

/**
 * Key used to identify an entry on the server and frontend.
 */
export interface EntryKey {
    readonly userID: string;
    readonly scheduleDate: ScheduleDate;
}

export interface EntryData {
    readonly amount: number;
}