<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import type { Contact } from '#lib/domain/messages.ts';

  import ShareSheet from './ShareSheet.svelte';

  import '#lib/components/styles/share.css';

  /**
   * web's ShareUserMenu: a Share button opening the one share sheet (`ShareSheet`, Owner Notes
   * Mockups §6): faces to send to, a search, and Copy link / Share via… / Quote in a post when
   * there is a link. The private event's Invite + has no link to share.
   */
  interface Props {
    menuTitle: string;
    /** Icon only, for tight action strips; the label stays as the accessible name. */
    compact?: boolean;
    /** The button's word (Detail Pages Cleanup §6: "Invite people" on the People page). */
    buttonLabel?: string;
    copyLinkUrl?: string | null;
    /** Title for the device share sheet and share links; defaults to the page title. */
    shareTitle?: string | null;
    /** "Project", "Thread", …: shown above what is shared. */
    kindLabel?: string;
    submitLabel?: string;
    recipientHint?: string;
    /** People offered before anyone types. */
    contacts?: Contact[];
    searchContacts: (query: string) => Promise<Contact[]>;
    /** Sends to one person; an error string stops the round and is shown. */
    submitShare: (contact: Contact) => Promise<string | null>;
    /** Quote in a post (Owner Notes Mockups §6). */
    createPost?: (() => void) | null;
  }

  let {
    menuTitle,
    compact = false,
    buttonLabel = 'Share',
    copyLinkUrl = null,
    shareTitle = null,
    kindLabel = '',
    submitLabel = 'Send',
    recipientHint = 'They get it as a message and a notification.',
    contacts = [],
    searchContacts,
    submitShare,
    createPost = null,
  }: Props = $props();

  let open = $state(false);
  let faces = $state<Contact[]>([]);

  // The faces: the caller's, else its search with nothing typed (the people you talk to most).
  async function show(): Promise<void> {
    open = true;
    if (contacts.length > 0 || faces.length > 0) return;
    try {
      faces = await searchContacts('');
    } catch {
      faces = [];
    }
  }
</script>

<span class="share-sheet">
  <div class="share-shell">
    <button
      aria-expanded={open}
      aria-label={menuTitle}
      class:active-toggle={open}
      class="share-button"
      type="button"
      onclick={() => (open ? (open = false) : void show())}
    >
      <svg aria-hidden="true" viewBox="0 0 24 24" width="18" height="18"
        ><path
          d="M12 3v12M7 8l5-5 5 5M5 13v6a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-6"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        /></svg
      >
      {#if !compact}<span class="share-button-label">{t(buttonLabel)}</span>{/if}
    </button>
  </div>
</span>

<ShareSheet
  {open}
  title={menuTitle}
  {kindLabel}
  subject={shareTitle ?? ''}
  url={copyLinkUrl}
  contacts={contacts.length > 0 ? contacts : faces}
  {searchContacts}
  {submitShare}
  {submitLabel}
  {recipientHint}
  onQuote={createPost}
  onClose={() => (open = false)}
/>
