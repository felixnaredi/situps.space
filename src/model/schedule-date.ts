export type UnboundedGregorianCalanderDate = {
  readonly year: number;
  readonly month: number;
  readonly day: number;
};

export type UnboundedYearDayOffset = {
  readonly year: number;
  readonly dayOffset: number;
};

export enum Weekday {
  SUNDAY = 0,
  MONDAY,
  TUESDAY,
  WEDNESDAY,
  THURDAY,
  FRIDAY,
  SATURDAY,
}

export class ScheduleDate {
  private _year: number;
  private _dayOffset: number;

  constructor(year: number, dayOffset: number) {
    while (dayOffset >= (isLeapYear(year) ? 366 : 365)) {
      dayOffset -= isLeapYear(year) ? 366 : 365;
      year += 1;
    }
    this._year = year;
    this._dayOffset = dayOffset;
  }

  static fromGregorian(date: UnboundedGregorianCalanderDate): ScheduleDate {
    // TODO:
    //   Throw error on invalid dates.
    const offset = isLeapYear(date.year)
      ? OFFSET_TO_MONTH_LEAP_YEAR
      : OFFSET_TO_MONTH;
    return new ScheduleDate(date.year, offset[date.month - 1] + date.day - 1);
  }

  public get year(): number {
    return this._year;
  }

  public get dayOffset(): number {
    return this._dayOffset;
  }

  public get month(): number {
    const offset = this.isLeapYear
      ? OFFSET_TO_MONTH_LEAP_YEAR
      : OFFSET_TO_MONTH;
    let i = 1;
    while (this._dayOffset >= offset[i]) {
      i++;
    }
    return i;
  }

  public get day(): number {
    const offset = this.isLeapYear
      ? OFFSET_TO_MONTH_LEAP_YEAR
      : OFFSET_TO_MONTH;
    return this._dayOffset - offset[this.month - 1] + 1;
  }

  /**
   * The weekday of the date.
   *
   * Sakamotos method.
   */
  public get weekday(): Weekday {
    const t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    const d = this.day;
    const m = this.month;
    const y = this.year - (m < 3 ? 1 : 0);
    return (
      (y +
        Math.floor(y / 4) -
        Math.floor(y / 100) +
        Math.floor(y / 400) +
        t[m - 1] +
        d) %
      7
    );
  }

  public get week(): number {
    return (
      mod(
        Math.floor(this.reversedToWeekday(Weekday.MONDAY).dayOffset / 7),
        52
      ) + 1
    );
  }

  public get isLeapYear(): boolean {
    return isLeapYear(this._year);
  }

  public before(other: ScheduleDate) {
    if (this._year == other._year) {
      return this._dayOffset < other._dayOffset;
    } else {
      return this._year < other._year;
    }
  }

  public reversedToWeekday(weekday: Weekday): ScheduleDate {
    return new ScheduleDate(
      this._year,
      this._dayOffset - mod(this.weekday - weekday, 7)
    );
  }
}

const OFFSET_TO_MONTH: Array<number> = [
  0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334,
];

const OFFSET_TO_MONTH_LEAP_YEAR: Array<number> = [
  0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335,
];

function isLeapYear(year: number): boolean {
  return year % 4 == 0 && (year % 400 == 0 || year % 100 != 0);
}

function mod(x: number, m: number): number {
  return ((x % m) + m) % m;
}
