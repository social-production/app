/**
 * web's stores/chatChrome: true while a detail chat is the open surface. On phones the app
 * shell then hides its top bar and bottom nav, so the chat fills the screen.
 */
export const chatChrome = $state<{ immersive: boolean }>({ immersive: false });
