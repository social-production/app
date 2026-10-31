<script lang="ts">
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import type { Contact } from '#lib/domain/messages.ts';

  import OverlaySheet from './OverlaySheet.svelte';
  import ShareOutside from './ShareOutside.svelte';

  import '#lib/components/styles/share.css';

  /**
   * web's ShareUserMenu: a Share button opening a sheet that shares outside the app (when
   * `copyLinkUrl` is set; `ShareOutside`), then searches people, collects chips and sends to
   * each. The private event's Invite + has no link to share.
   */
  interface Props {
    menuTitle: string;
    /** Icon only, for tight action strips; the label stays as the accessible name. */
    compact?: boolean;
    copyLinkUrl?: string | null;
    /** Title for the device share sheet and share links; defaults to the page title. */
    shareTitle?: string | null;
    placeholder?: string;
    submitLabel?: string;
    recipientHint?: string;
    /** People offered before anyone types. */
    contacts?: Contact[];
    searchContacts: (query: string) => Promise<Contact[]>;
    /** Sends to one person; an error string stops the round and is shown. */
    submitShare: (contact: Contact) => Promise<string | null>;
  }

  let {
    menuTitle,
    compact = false,
    copyLinkUrl = null,
    shareTitle = null,
    placeholder = 'Type a username',
    submitLabel = 'Send',
    recipientHint = 'They get it as a message and a notification.',
    contacts = [],
    searchContacts,
    submitShare,
  }: Props = $props();

  let open = $state(false);
  let query = $state('');
  let pending = $state(false);
  let feedback = $state('');
  let selected = $state<Contact[]>([]);
  let live = $state<Contact[]>([]);
  let searching = $state(false);
  let requestId = 0;

  const needle = $derived(query.trim().toLowerCase());
  const picked = $derived(new Set(selected.map((p) => p.agent)));
  const shown = $derived(
    needle
      ? (live.length > 0 ? live : contacts)
          .filter((c) => c.name.toLowerCase().includes(needle))
          .slice(0, 8)
      : contacts.slice(0, 5),
  );

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
    selected = picked.has(contact.agent)
      ? selected.filter((p) => p.agent !== contact.agent)
      : [...selected, contact];
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

  function onKeydown(event: KeyboardEvent): void {
    if (event.key !== 'Enter') return;
    event.preventDefault();
    void submit();
  }

  function close(): void {
    open = false;
    feedback = '';
    query = '';
    live = [];
    selected = [];
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
      onclick={() => (open ? close() : (open = true))}
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
      {#if !compact}<span class="share-button-label">Share</span>{/if}
    </button>
  </div>
</span>

<OverlaySheet
  {open}
  labelledById="share-sheet-title"
  title={menuTitle}
  rootClass="share-sheet"
  onClose={close}
>
  {#snippet toolbar()}
    {#if copyLinkUrl}
      <ShareOutside url={copyLinkUrl} title={shareTitle ?? document.title} />
    {/if}
    <div class="share-search">
      <span class="share-out-label">Send to people here</span>
      <label class="search-box" for="share-people-search">
        <svg aria-hidden="true" viewBox="0 0 24 24" width="18" height="18"
          ><circle cx="11" cy="11" r="7" fill="none" stroke="currentColor" stroke-width="2" /><path
            d="M20 20l-4-4"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
          /></svg
        >
        <span class="sr-only">{menuTitle}</span>
        <input
          id="share-people-search"
          bind:value={query}
          maxlength="64"
          {placeholder}
          type="search"
          autocomplete="off"
          oninput={() => void onInput()}
          onkeydown={onKeydown}
        />
      </label>
    </div>
  {/snippet}
  {#if selected.length > 0}
    <div class="selected-row" aria-label="Selected recipients">
      {#each selected as person (person.agent)}
        <button class="selected-chip" type="button" onclick={() => toggle(person)}>
          <AvatarBadge size="sm" name={person.name} />
          <span>{person.name}</span>
          <span class="chip-remove" aria-label={`Remove ${person.name}`}>×</span>
        </button>
      {/each}
    </div>
  {/if}
  {#if shown.length > 0}
    {#if !needle}<p class="list-label">Suggested</p>{/if}
    <div class="contact-list">
      {#each shown as contact (contact.agent)}
        <button
          class:selected={picked.has(contact.agent)}
          class="contact-row"
          type="button"
          aria-pressed={picked.has(contact.agent)}
          onclick={() => toggle(contact)}
        >
          <AvatarBadge size="sm" name={contact.name} />
          <span class="contact-name">{contact.name}</span>
          <span class="check" aria-hidden="true">
            {#if picked.has(contact.agent)}
              <svg viewBox="0 0 24 24" width="14" height="14"
                ><path
                  d="M5 12l5 5 9-10"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="3"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                /></svg
              >
            {/if}
          </span>
        </button>
      {/each}
    </div>
  {:else if needle && searching}
    <p class="empty-row">Searching…</p>
  {:else if needle}
    <p class="empty-row">No one matches “{query.trim()}”.</p>
  {:else if selected.length === 0}
    <p class="empty-row">Search for people by username. {recipientHint}</p>
  {/if}
  {#if feedback}<p class="feedback">{feedback}</p>{/if}
  {#snippet footer()}
    <div class="sheet-actions">
      <button
        class="sheet-submit"
        disabled={selected.length === 0 || pending}
        type="button"
        onclick={() => void submit()}
      >
        {pending
          ? 'Sending…'
          : selected.length > 1
            ? `${submitLabel} to ${selected.length}`
            : submitLabel}
      </button>
    </div>
  {/snippet}
</OverlaySheet>
