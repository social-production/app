import type { ShareSubject } from '#lib/components/overlay/shareLink.ts';

/** What the shell's share sheet shows (`ShareHost`); null while it is closed. */
export const shareHost = $state<{ subject: ShareSubject | null }>({ subject: null });
