import { ScheduleDate, Weekday } from "./schedule-date";
import { InclusiveScheduleDateRange } from "./schedule-date-range";

export type AnchorDate = ScheduleDate | "now";

export interface ScheduleDateRangeConfigOptions {
  readonly anchorDate: AnchorDate;
  readonly previousWeeksAmount: number;
  readonly upcomingWeeksAmount: number;
}

export function ScheduleDateRangeWithConfig(
  options: ScheduleDateRangeConfigOptions
): InclusiveScheduleDateRange {
  const anchorDate = (
    options.anchorDate == "now"
      ? ScheduleDate.fromDate(new Date())
      : options.anchorDate
  ).reversedToWeekday(Weekday.MONDAY);

  const begin = new ScheduleDate(
    anchorDate.year,
    anchorDate.dayOffset - options.previousWeeksAmount * 7
  ).reversedToWeekday(Weekday.MONDAY);

  const end = new ScheduleDate(
    anchorDate.year,
    anchorDate.dayOffset + options.upcomingWeeksAmount * 7
  ).reversedToWeekday(Weekday.SUNDAY);

  return new InclusiveScheduleDateRange(begin, end);
}
