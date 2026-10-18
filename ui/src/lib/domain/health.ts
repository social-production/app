/** Mirrors `zomes/health`. Hashes cross the UI boundary as base64 strings. */
export interface Heartbeat {
  hash: string;
}

export const NOTE_MAX = 280; // sp_core::limits::NOTE_MAX
