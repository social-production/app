<script lang="ts">
  import { tick } from 'svelte';

  import { MESSAGE_MAX, type ChatLine } from '#lib/domain/chat.ts';
  import type { Mentionable } from '#lib/domain/mentions.ts';

  import AttachButtons from './AttachButtons.svelte';
  import { ComposerFiles } from './composerFiles.svelte.ts';
  import MentionMenu from './MentionMenu.svelte';
  import { activeMention, insertMention } from './mentions.ts';

  /** web LiveChatPanel's composer card: the reply bar, photos and files to send, the growing
   * textarea and Send. */
  interface Props {
    placeholder: string;
    submitLabel: string;
    replyTarget: ChatLine | null;
    /** Your own message being edited (web's editingMessage): its text fills the field. */
    editing?: ChatLine | null;
    paused: boolean;
    error: string;
    /** Resolves once sent; throws to keep the draft (and the files). */
    onSubmit: (body: string, files: File[]) => Promise<void>;
    /** web's `allowAttachments` (direct and group chats, ADR 0049). */
    attachments?: boolean;
    onCancelReply: () => void;
    onCancelEdit?: () => void;
    onFocus: () => void;
    /** Set where web offers mentions (project chats): people whose name matches. */
    mentionSearch?: ((query: string) => Promise<Mentionable[]>) | null | undefined;
    /** Guests: web shows "Sign in" in place of Send, and sending asks them to sign in. */
    signIn?: (() => void) | null | undefined;
  }

  let p: Props = $props();

  const COMPOSER_MAX_HEIGHT = 132;

  let draft = $state('');
  let pending = $state(false);
  let field = $state<HTMLTextAreaElement>();
  const files = new ComposerFiles();
  const canSend = $derived(Boolean(draft.trim()) || files.items.length > 0);
  let mentionResults = $state<Mentionable[]>([]);
  let mentionStart = -1;
  let mentionCursor = 0;
  let mentionTimer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    if (field && draft === '') field.style.height = '';
  });

  // Starting an edit puts the message's text in the field.
  $effect(() => {
    const message = p.editing;
    if (!message) return;
    draft = message.body;
    void Promise.resolve().then(() => {
      grow();
      field?.focus();
    });
  });

  function grow(): void {
    if (!field) return;
    field.style.height = 'auto';
    field.style.height = `${Math.min(field.scrollHeight, COMPOSER_MAX_HEIGHT)}px`;
  }

  async function submit(): Promise<void> {
    if (p.signIn) return p.signIn();
    const body = draft.trim();
    // An edit changes only the text, as on web.
    const picked = p.editing ? [] : files.files;
    if ((!body && picked.length === 0) || pending) return;
    if (p.editing && !body) return;
    draft = '';
    clearMentions();
    pending = true;
    try {
      await p.onSubmit(body, picked);
      if (picked.length) files.release();
      files.error = '';
    } catch (e) {
      draft = body;
      if (e instanceof Error && e.message) files.error = e.message;
    } finally {
      pending = false;
    }
  }

  function clearMentions(): void {
    clearTimeout(mentionTimer);
    mentionResults = [];
    mentionStart = -1;
  }

  /** web's syncMentions: look names up shortly after the "@…" before the caret changes. */
  function syncMentions(): void {
    const search = p.mentionSearch;
    if (!search || !field) return clearMentions();
    const cursor = field.selectionStart ?? draft.length;
    const found = activeMention(draft, cursor);
    if (!found) return clearMentions();
    mentionStart = found.start;
    mentionCursor = cursor;
    clearTimeout(mentionTimer);
    mentionTimer = setTimeout(() => {
      void search(found.query).then(
        (items) => {
          if (mentionStart === found.start) mentionResults = items;
        },
        () => clearMentions(),
      );
    }, 160);
  }

  async function chooseMention(name: string): Promise<void> {
    const inserted = insertMention(draft, mentionStart, mentionCursor, name);
    draft = inserted.value;
    clearMentions();
    await tick();
    field?.focus();
    field?.setSelectionRange(inserted.caret, inserted.caret);
  }

  function keydown(event: KeyboardEvent): void {
    if (event.key === 'Escape' && mentionResults.length > 0) {
      event.preventDefault();
      clearMentions();
      return;
    }
    if (event.key !== 'Enter' || event.shiftKey) return;
    const [first] = mentionResults;
    if (first) {
      event.preventDefault();
      void chooseMention(first.name);
      return;
    }
    // On phones Enter adds a line, as in other messaging apps; the send button sends.
    if (window.matchMedia('(pointer: coarse)').matches) return;
    event.preventDefault();
    void submit();
  }
</script>

{#snippet cancel(label: string, onclick: () => void)}
  <button type="button" aria-label={label} title={label} {onclick}>
    <svg aria-hidden="true" viewBox="0 0 24 24" width="18" height="18">
      <path
        d="M6 6l12 12M18 6L6 18"
        fill="none"
        stroke="currentColor"
        stroke-width="2.2"
        stroke-linecap="round"
      />
    </svg>
  </button>
{/snippet}

<div class="composer-card">
  {#if p.editing}
    <div class="composer-context">
      <svg
        class="composer-context-icon"
        aria-hidden="true"
        viewBox="0 0 24 24"
        width="20"
        height="20"
      >
        <path
          d="M4 17.5V20h2.5l9.9-9.9-2.5-2.5L4 17.5zm15.7-9.8a1 1 0 0 0 0-1.4l-2-2a1 1 0 0 0-1.4 0l-1.5 1.5 3.4 3.4 1.5-1.5z"
          fill="currentColor"
        />
      </svg>
      <div class="composer-context-text"><strong>Edit message</strong></div>
      {@render cancel('Cancel edit', () => {
        draft = '';
        p.onCancelEdit?.();
      })}
    </div>
  {:else if p.replyTarget}
    <div class="composer-context">
      <svg
        class="composer-context-icon"
        aria-hidden="true"
        viewBox="0 0 24 24"
        width="20"
        height="20"
      >
        <path d="M10 8V4l-7 7 7 7v-4c5 0 8 1.5 11 6-1-6-4-11-11-12z" fill="currentColor" />
      </svg>
      <div class="composer-context-text">
        <strong>Reply to {p.replyTarget.authorName}</strong>
        <p>{p.replyTarget.body}</p>
      </div>
      {@render cancel('Cancel reply', p.onCancelReply)}
    </div>
  {/if}
  {#if p.attachments && files.items.length > 0}
    <div class="attach-previews">
      {#each files.items as item (item.id)}
        <div class="attach-preview">
          {#if item.previewUrl}<img alt="" src={item.previewUrl} />{:else}<span
              >{item.file.name}</span
            >{/if}
          <button
            aria-label="Remove attachment"
            type="button"
            onclick={() => files.release(item.id)}>Remove</button
          >
        </div>
      {/each}
    </div>
  {/if}
  {#if p.error || files.error}
    <p class="attach-error" role="alert">{p.error || files.error}</p>
  {/if}
  {#if p.paused}
    <p class="composer-paused" role="status">
      This account is read-only: a community review found it's a bot.
    </p>
  {:else}
    <div class="composer-row">
      {#if p.attachments}<AttachButtons {files} part="buttons" />{/if}
      <div class="composer-input-shell">
        <MentionMenu results={mentionResults} onSelect={(name) => void chooseMention(name)} />
        <textarea
          bind:this={field}
          bind:value={draft}
          maxlength={MESSAGE_MAX}
          onfocus={p.onFocus}
          oninput={() => {
            syncMentions();
            grow();
          }}
          onkeydown={keydown}
          placeholder={p.placeholder}
          aria-label={p.placeholder}
          rows="1"></textarea>
      </div>
      {#if p.signIn}
        <button class="primary-button" type="button" onclick={p.signIn}>Sign in</button>
      {:else if p.editing}
        <button
          class="send-button save"
          disabled={pending || !canSend}
          type="button"
          onclick={submit}>Save</button
        >
      {:else}
        <button
          class="send-button"
          aria-label={p.submitLabel}
          title={p.submitLabel}
          disabled={pending || !canSend}
          type="button"
          onclick={submit}
        >
          <svg aria-hidden="true" viewBox="0 0 24 24" width="20" height="20">
            <path d="M3.4 20.4 21 12 3.4 3.6 3.4 10l12.6 2-12.6 2z" fill="currentColor" />
          </svg>
        </button>
      {/if}
    </div>
  {/if}
  {#if p.attachments && !p.paused}<AttachButtons {files} part="inputs" />{/if}
</div>
