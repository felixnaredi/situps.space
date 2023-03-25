export type UnboundedGregorianCalanderDate = {
  readonly year: number;
  readonly month: number;
  readonly day: number;
};

export type UnboundedYearDayOffset = {
  readonly year: number;
  readonly dayOffset: number;
};

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
