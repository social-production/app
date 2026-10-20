<script lang="ts">
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import type { ChatKind, Contact, NewChat } from '#lib/domain/messages.ts';

  import SuggestionList from './SuggestionList.svelte';

  import '#lib/components/styles/messages.css';

  /** web's ComposeMessageSheet: a direct message to one person, or a named group. */
  interface Props {
    open: boolean;
    /** Someone already chosen (web's `prefillUsername`, from a profile's Message button). */
    prefill?: Contact | null;
    search: (query: string) => Promise<Contact[]>;
    /** Starts the chat; a thrown error is shown and keeps the draft. */
    onSend: (chat: NewChat) => Promise<void>;
    onClose: () => void;
  }

  let { open, prefill = null, search, onSend, onClose }: Props = $props();

  let mode = $state<ChatKind>('direct');
  let recipientDraft = $state('');
  let direct = $state<Contact | null>(null);
  let titleDraft = $state('');
  let memberDraft = $state('');
  let group = $state<Contact[]>([]);
  let body = $state('');
  let error = $state('');
  let submitting = $state(false);
  let suggestions = $state<Contact[]>([]);
  let searching = $state(false);
  let searchId = 0;

  const query = $derived(mode === 'direct' ? recipientDraft : memberDraft);
  const directItems = $derived(suggestions);
  const groupItems = $derived(suggestions.filter((c) => !group.some((g) => g.agent === c.agent)));

  // Opening starts fresh (with the prefilled person, if any), as web's resetComposer does.
  $effect(() => {
    if (!open) return;
    mode = 'direct';
    recipientDraft = memberDraft = titleDraft = body = error = '';
    direct = prefill;
    group = [];
    submitting = false;
  });

  $effect(() => {
    const q = query.trim();
    const id = ++searchId;
    if (!q) {
      suggestions = [];
      return;
    }
    searching = true;
    search(q)
      .then((found) => id === searchId && (suggestions = found))
      .catch(() => id === searchId && (suggestions = []))
      .finally(() => id === searchId && (searching = false));
  });

  function chooseDirect(c: Contact): void {
    direct = c;
    recipientDraft = '';
    error = '';
  }

  function chooseMember(c: Contact): void {
    if (!group.some((g) => g.agent === c.agent)) group = [...group, c];
    memberDraft = '';
    error = '';
  }

  function switchTo(next: ChatKind): void {
    mode = next;
    error = '';
    // Keep the person already chosen when switching, as web does.
    if (next === 'group' && direct && !group.some((g) => g.agent === direct?.agent))
      group = [...group, direct];
    if (next === 'direct' && !direct && group.length === 1) direct = group[0] ?? null;
  }

  function onKeydown(event: KeyboardEvent, pick: () => void): void {
    if (event.key === 'Enter') {
      event.preventDefault();
      pick();
    }
  }

  async function submit(): Promise<void> {
    if (submitting) return;
    const text = body.trim();
    const members = mode === 'direct' ? (direct ? [direct] : []) : group;
    if (mode === 'direct' && (!direct || !text)) {
      error = 'Choose a username and write a message.';
      return;
    }
    if (!text) {
      error = 'Write a message.';
      return;
    }
    if (members.length === 0) {
      error = 'Group conversation requires at least one other user';
      return;
    }
    submitting = true;
    error = '';
    try {
      await onSend({ kind: mode, members, title: titleDraft.trim(), body: text });
    } catch (e) {
      error = e instanceof Error ? e.message : 'The chat could not be started.';
    } finally {
      submitting = false;
    }
  }
</script>

<OverlaySheet
  {open}
  labelledById="compose-message-sheet-title"
  title="New message"
  rootClass="compose-sheet"
  {onClose}
>
  <div class="compose-form">
    <div class="composer-mode-row" role="tablist" aria-label="Message type">
      {#each [['direct', 'Direct'], ['group', 'Group']] as const as [kind, label] (kind)}
        <button
          aria-selected={mode === kind}
          class:active={mode === kind}
          class="composer-mode"
          role="tab"
          type="button"
          onclick={() => switchTo(kind)}>{label}</button
        >
      {/each}
    </div>

    {#if mode === 'direct'}
      <div class="composer-field">
        {#if direct}
          <div class="selected-row" aria-label="Selected person">
            <button class="selected-chip" type="button" onclick={() => (direct = null)}>
              <AvatarBadge size="sm" name={direct.name} />
              <span>{direct.name}</span>
              <span class="chip-remove" aria-hidden="true">×</span>
            </button>
          </div>
        {:else}
          <label>
            <span class="sr-only">To</span>
            <input
              bind:value={recipientDraft}
              onkeydown={(e) => {
                const [only, ...rest] = directItems;
                if (only && rest.length === 0) onKeydown(e, () => chooseDirect(only));
              }}
              placeholder="Username"
              type="text"
            />
          </label>
          <SuggestionList
            items={directItems}
            query={recipientDraft}
            emptyCopy={searching ? 'Searching…' : 'No matches.'}
            onSelect={chooseDirect}
          />
        {/if}
      </div>
    {:else}
      <label class="composer-field">
        <span class="sr-only">Group chat name</span>
        <input bind:value={titleDraft} placeholder="Group name" type="text" maxlength="200" />
      </label>
      <div class="composer-field">
        <label>
          <span class="sr-only">Add members</span>
          <input
            bind:value={memberDraft}
            onkeydown={(e) => {
              const [first] = groupItems;
              if (first) onKeydown(e, () => chooseMember(first));
            }}
            placeholder="Add members"
            type="text"
          />
        </label>
        {#if group.length > 0}
          <div class="selected-row" aria-label="Selected members">
            {#each group as member (member.agent)}
              <button
                class="selected-chip"
                type="button"
                onclick={() => (group = group.filter((g) => g.agent !== member.agent))}
              >
                <AvatarBadge size="sm" name={member.name} />
                <span>{member.name}</span>
                <span class="chip-remove" aria-hidden="true">×</span>
              </button>
            {/each}
          </div>
        {/if}
        <SuggestionList
          items={groupItems}
          query={memberDraft}
          emptyCopy={searching ? 'Searching…' : 'No matches.'}
          onSelect={chooseMember}
        />
      </div>
    {/if}

    <label class="composer-field grow">
      <span class="sr-only">Message</span>
      <textarea
        bind:value={body}
        onkeydown={(e) => !e.shiftKey && onKeydown(e, () => void submit())}
        placeholder="Write a message…"
        rows="4"></textarea>
    </label>
    {#if error}<p class="composer-feedback">{error}</p>{/if}
  </div>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-cancel" type="button" onclick={onClose}>Cancel</button>
      <button
        class="sheet-submit"
        disabled={submitting}
        type="button"
        onclick={() => void submit()}
      >
        {submitting ? 'Sending…' : 'Send'}
      </button>
    </div>
  {/snippet}
</OverlaySheet>
