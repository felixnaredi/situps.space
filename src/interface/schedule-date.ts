export interface ScheduleDateData {
  readonly year: number;
  readonly month: number;
  readonly day: number;
}

export class ScheduleDate {
  private data: ScheduleDateData;

  constructor(date: ScheduleDateData) {
    this.data = date;
  }

  public get year(): number {
    return this.data.year;
  }
  public get month(): number {
    return this.data.month;
  }
  public get day(): number {
    return this.data.day;
  }

  public get isLeapYear(): boolean {
    return (
      this.data.year % 4 == 0 &&
      (this.data.year % 400 == 0 || this.data.year % 100 != 0)
    );
  }

  public before(other: ScheduleDate) {
    if (this.year != other.year) {
      return this.year < other.year;
    }
    if (this.month != other.month) {
      return this.month < other.month;
    }
    return this.day < other.day;
  }
}

/*
const LAST_DAY_OF_MONTH: Record<number, number> = {
  1: 31,
  2: 28,
  3: 31,
  4: 30,
  5: 31,
  6: 30,
  7: 31,
  8: 31,
  9: 30,
  10: 31,
  11: 30,
  12: 31,
};

export function dayOfYear(date: ScheduleDate): number {
  const daysOfPassedMonths = Array.from(new Array(date.month - 1))
    .map((i) => LAST_DAY_OF_MONTH[i + 1])
    .reduce((acc, x) => acc + x);
  const leapDay = date.month > 2 && isLeapYear(date) ? 1 : 0;

  return daysOfPassedMonths + leapDay + date.day;
}
*/
