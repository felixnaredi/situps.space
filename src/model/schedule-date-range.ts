import { dateComesBefore, ScheduleDate } from "../interface/schedule-date";

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

function isLeapYear(year: number): boolean {
  return year % 4 == 0 && (year % 400 == 0 || year % 100 != 0);
}

class ScheduleDateIterator implements Iterator<ScheduleDate> {
  private current: ScheduleDate;

  constructor(current: ScheduleDate) {
    this.current = current;
  }

  public next(): IteratorResult<ScheduleDate> {
    const lastDayOfMonth = LAST_DAY_OF_MONTH[this.current.month];
    const old = this.current;

    if (this.current.day >= lastDayOfMonth) {
      //
      // Check if the month should be incremented.
      //
      if (
        this.current.month != 2 ||
        !(isLeapYear(this.current.year) && this.current.day == 28)
      ) {
        //
        // Check if the year should be incremented.
        //
        if (this.current.month == 12) {
          this.current = {
            day: 1,
            month: 1,
            year: this.current.year + 1,
          };
        } else {
          this.current = {
            day: 1,
            month: this.current.month + 1,
            year: this.current.year,
          };
        }
      } else {
        this.current = {
          day: this.current.day + 1,
          month: this.current.month,
          year: this.current.year,
        };
      }
    } else {
      this.current = {
        day: this.current.day + 1,
        month: this.current.month,
        year: this.current.year,
      };
    }
    //
    // Return the incremented state.
    //
    return {
      done: false,
      value: old,
    };
  }
}

export class InclusiveScheduleDateRange implements Iterable<ScheduleDate> {
  private readonly begin: ScheduleDate;
  private readonly end: ScheduleDate;

  constructor(begin: ScheduleDate, end: ScheduleDate) {
    this.begin = begin;
    this.end = end;
  }

  public [Symbol.iterator](): Iterator<ScheduleDate> {
    const it = new ScheduleDateIterator(this.end);
    it.next();
    const end = it.next().value;

    let current = new ScheduleDateIterator(this.begin);

    return {
      next: () => {
        const value = current.next().value;
        if (dateComesBefore(value, end)) {
          return {
            done: false,
            value,
          };
        } else {
          return {
            done: true,
            value: null,
          };
        }
      },
    };
  }
}
