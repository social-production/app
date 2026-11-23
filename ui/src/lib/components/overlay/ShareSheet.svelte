<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import type { Contact } from '#lib/domain/messages.ts';

  import OverlaySheet from './OverlaySheet.svelte';
  import ShareActions from './ShareActions.svelte';

  import '#lib/components/styles/ux/sharesheet.css';

  /**
   * One share sheet everywhere (Owner Notes Mockups §6, approved 2026-11-24): what is shared on
   * top, the people you talk to most as faces (tap to pick, then Send), a search for anyone
   * else, then Copy link, Share via… (the phone's own sheet) and Quote in a post.
   */
  interface Props {
    open: boolean;
    title: string;
    /** "Project", "Thread", …: the line above what is shared. */
    kindLabel?: string;
    subject?: string;
    url?: string | null;
    contacts?: Contact[];
    searchContacts: (query: string) => Promise<Contact[]>;
    /** Sends to one person; an error string stops the round and is shown. */
    submitShare: (contact: Contact) => Promise<string | null>;
    submitLabel?: string;
    recipientHint?: string;
    /** Quote in a post (a post with your words over the shared card). */
    onQuote?: (() => void) | null;
    onClose: () => void;
  }

  let {
    open,
    title,
    kindLabel = '',
    subject = '',
    url = null,
    contacts = [],
    searchContacts,
    submitShare,
    submitLabel = 'Send',
    recipientHint = 'They get it as a message and a notification.',
    onQuote = null,
    onClose,
  }: Props = $props();

  let query = $state('');
  let pending = $state(false);
  let feedback = $state('');
  let selected = $state<Contact[]>([]);
  let live = $state<Contact[]>([]);
  let searching = $state(false);
  let requestId = 0;

  const needle = $derived(query.trim().toLowerCase());
  const picked = $derived(selected.map((p) => p.agent));
  // Faces: what the search finds, else the people you talk to most; picked people stay shown.
  const faces = $derived.by(() => {
    const base = needle
      ? (live.length > 0 ? live : contacts).filter((c) => c.name.toLowerCase().includes(needle))
      : contacts;
    const rest = base.filter((c) => !picked.includes(c.agent));
    return [...(needle ? [] : selected), ...rest].slice(0, 12);
  });

  async function onInput(): Promise<void> {
    const q = query.trim();
    if (!q) {
      live = [];
      return;
    }
    const id = ++requestId;
    searching = true;
    try {
      const found = await searchContacts(q);
      if (id === requestId) live = found;
    } catch {
      if (id === requestId) live = [];
    } finally {
      if (id === requestId) searching = false;
    }
  }

  function toggle(contact: Contact): void {
    selected = picked.includes(contact.agent)
      ? selected.filter((p) => p.agent !== contact.agent)
      : [...selected, contact];
    query = '';
    live = [];
  }

  async function submit(): Promise<void> {
    if (selected.length === 0 || pending) return;
    pending = true;
    feedback = '';
    try {
      for (const person of selected) {
        const error = await submitShare(person);
        if (error) {
          feedback = error;
          return;
        }
      }
      close();
    } finally {
      pending = false;
    }
  }

  function close(): void {
    feedback = '';
    query = '';
    live = [];
    selected = [];
    onClose();
  }
</script>

<OverlaySheet
  {open}
  labelledById="share-panel-title"
  {title}
  rootClass="share-panel"
  onClose={close}
>
  {#if subject}
    <div class="share-subject-card">
      {#if kindLabel}<span class="share-subject-kind">{t(kindLabel)}</span>{/if}
      <strong>{subject}</strong>
    </div>
  {/if}
  <p class="share-label">{t('Send to')}</p>
  <label class="share-find">
    <svg aria-hidden="true" viewBox="0 0 24 24" width="16" height="16"
      ><circle cx="11" cy="11" r="7" fill="none" stroke="currentColor" stroke-width="2" /><path
        d="M20 20l-4-4"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
      /></svg
    >
    <span class="sr-only">{t('Search people')}</span>
    <input
      bind:value={query}
      maxlength="64"
      placeholder={t('Search people')}
      type="search"
      autocomplete="off"
      oninput={() => void onInput()}
      onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), void submit())}
    />
  </label>
  {#if faces.length > 0}
    <div class="share-faces" role="group" aria-label={t('People')}>
      {#each faces as contact (contact.agent)}
        <button
          class="share-face"
          class:picked={picked.includes(contact.agent)}
          type="button"
          aria-pressed={picked.includes(contact.agent)}
          onclick={() => toggle(contact)}
        >
          <span class="share-face-avatar">
            <AvatarBadge size="md" name={contact.name} agent={contact.agent} />
            {#if picked.includes(contact.agent)}
              <span class="share-face-check" aria-hidden="true">
                <svg viewBox="0 0 24 24" width="12" height="12"
                  ><path
                    d="M5 12l5 5 9-10"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="3.2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  /></svg
                >
              </span>
            {/if}
          </span>
          <span class="share-face-name">{contact.name}</span>
        </button>
      {/each}
    </div>
  {:else if needle && searching}
    <p class="share-empty">{t('Searching…')}</p>
  {:else if needle}
    <p class="share-empty">{t('No one matches “{query}”.', { query: query.trim() })}</p>
  {:else}
    <p class="share-empty">{t(recipientHint)}</p>
  {/if}
  {#if feedback}<p class="share-feedback" role="alert">{feedback}</p>{/if}
  {#if url}
    <ShareActions
      {url}
      title={subject || title}
      onQuote={onQuote ? () => (close(), onQuote?.()) : null}
    />
  {/if}
  {#snippet footer()}
    {#if selected.length > 0}
      <div class="sheet-actions">
        <button class="sheet-submit" disabled={pending} type="button" onclick={() => void submit()}>
          {pending
            ? t('Sending…')
            : selected.length > 1
              ? t('{label} to {n}', { label: t(submitLabel), n: selected.length })
              : t(submitLabel)}
        </button>
      </div>
    {/if}
  {/snippet}
</OverlaySheet>
