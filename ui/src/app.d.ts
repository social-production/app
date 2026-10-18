declare global {
  interface Window {
    /** Injected by Holochain's Tauri runtime when running inside the native app. */
    __HC_TAURI_HOLOCHAIN__?: unknown;
  }
}

export {};
