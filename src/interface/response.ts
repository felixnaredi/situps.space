import { EntryData, EntryKey } from "./entry";


export interface EntryEventGetResponse {
    readonly entryKey: EntryKey,
    readonly entryData: null | EntryData;
}

export interface EntryEventStateChange {
    readonly entryKey: EntryKey;
    readonly oldValue: null | EntryData;
    // TODO:
    //   Should `newValue` be nullable?
    readonly newValue: EntryData;
}
