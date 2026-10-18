declare global {
  interface Window {
    /** Injected by the native (Tauri) shell: URL of its embedded gateway on localhost. */
    __SP_LOCAL_GATEWAY__?: string;
  }
}

export {};
