
export interface ScheduleDate {
  readonly year: number;
  readonly month: number;
  readonly day: number;
}

export function dateComesBefore(first: ScheduleDate, second: ScheduleDate) {
  if (first.year != second.year) {
    return first.year < second.year;
  }
  if (first.month != second.month) {
    return first.month < second.month;
  }
  return first.day < second.day;
}