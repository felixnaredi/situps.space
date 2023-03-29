/**
 * Key used to identify an entry on the server and frontend.
 */
export type EntryKey = {
  readonly userId: string;
  readonly scheduleDate: {
    readonly year: number;
    readonly month: number;
    readonly day: number;
  };
};

export function EntryKeyIdentifier(entryKey: EntryKey): string {
  return JSON.stringify(entryKey);
}

export interface EntryData {
  readonly amount: number;
}
