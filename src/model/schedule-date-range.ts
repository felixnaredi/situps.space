import { ScheduleDate } from "./schedule-date";

class ScheduleDateIterator implements Iterator<ScheduleDate> {
  private current: ScheduleDate;

  constructor(current: ScheduleDate) {
    this.current = current;
  }

  public next(): IteratorResult<ScheduleDate> {
    const old = this.current;
    this.current = new ScheduleDate(
      this.current.year,
      this.current.dayOffset + 1
    );
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

        if (value.before(end)) {
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
