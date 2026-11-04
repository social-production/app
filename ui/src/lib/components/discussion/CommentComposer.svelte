<script lang="ts">
  import { tick } from 'svelte';

  import MentionMenu from '#lib/components/chat/MentionMenu.svelte';
  import { activeMention, insertMention } from '#lib/components/chat/mentions.ts';
  import type { Mentionable } from '#lib/domain/mentions.ts';

  /**
   * web's CommentComposer: a growing textarea (Enter sends, Shift+Enter breaks the line) with a
   * send button once there is text, and web's @mention menu; signed out it is a button asking
   * you to sign in.
   */
  interface Props {
    value: string;
    placeholder?: string;
    submitLabel?: string;
    signInMessage?: string;
    signedIn: boolean;
    /** Asks the person to sign in (the app's auth notice) with this message. */
    onSignIn: (message: string) => void;
    onSubmit: () => void;
    /** People whose name matches the "@…" being typed (web's searchPeopleSuggestions). */
    mentionSearch?: ((query: string) => Promise<Mentionable[]>) | null | undefined;
  }

  let {
    value = $bindable(''),
    placeholder = 'Write a comment...',
    submitLabel = 'Post comment',
    signInMessage = 'Sign in to comment.',
    signedIn,
    onSignIn,
    onSubmit,
    mentionSearch = null,
  }: Props = $props();

  const MAX_HEIGHT = 240;
  let field = $state<HTMLTextAreaElement>();
  let scrollable = $state(false);
  const canSubmit = $derived(value.trim().length > 0);
  let mentionResults = $state<Mentionable[]>([]);
  let mentionStart = -1;
  let mentionCursor = 0;
  let mentionTimer: ReturnType<typeof setTimeout> | undefined;

  function clearMentions(): void {
    clearTimeout(mentionTimer);
    mentionResults = [];
    mentionStart = -1;
  }

  /** web's syncMentions: look names up shortly after the "@…" before the caret changes. */
  function syncMentions(): void {
    if (!mentionSearch || !field) return clearMentions();
    const search = mentionSearch;
    const cursor = field.selectionStart ?? value.length;
    const found = activeMention(value, cursor);
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
    const inserted = insertMention(value, mentionStart, mentionCursor, name);
    value = inserted.value;
    clearMentions();
    await tick();
    field?.focus();
    field?.setSelectionRange(inserted.caret, inserted.caret);
  }

  function resize(): void {
    if (!field) return;
    field.style.height = 'auto';
    field.style.height = `${Math.min(field.scrollHeight, MAX_HEIGHT)}px`;
    scrollable = field.scrollHeight > MAX_HEIGHT;
  }

  async function submit(): Promise<void> {
    if (!signedIn) {
      onSignIn(signInMessage);
      return;
    }
    if (!value.trim()) return;
    clearMentions();
    onSubmit();
    await tick();
    resize();
  }

  function keydown(event: KeyboardEvent): void {
    if (event.key === 'Escape' && mentionResults.length > 0) {
      event.preventDefault();
      clearMentions();
      return;
    }
    if (event.key === 'Enter' && !event.shiftKey) {
      const [first] = mentionResults;
      if (first) {
        event.preventDefault();
        void chooseMention(first.name);
        return;
      }
      event.preventDefault();
      void submit();
    }
  }

  $effect(() => {
    void value;
    resize();
  });
</script>

<div class="composer-field">
  <MentionMenu results={mentionResults} onSelect={(name) => void chooseMention(name)} />
  {#if signedIn}
    <textarea
      bind:this={field}
      bind:value
      class="composer-input"
      class:scrollable
      {placeholder}
      aria-label={placeholder}
      rows="1"
      oninput={() => {
        syncMentions();
        resize();
      }}
      onkeydown={keydown}></textarea>
  {:else}
    <button
      class="composer-input composer-signin"
      type="button"
      onclick={() => onSignIn(signInMessage)}
    >
      {placeholder}
    </button>
  {/if}
  {#if signedIn && canSubmit}
    <button
      aria-label={submitLabel}
      title={submitLabel}
      class="composer-send"
      type="button"
      onclick={() => void submit()}
    >
      <svg aria-hidden="true" viewBox="0 0 24 24">
        <path
          d="M5 12.5 18.5 6 12 19.5V12.5H5Z"
          fill="currentColor"
          stroke="currentColor"
          stroke-width="1.2"
          stroke-linejoin="round"
        />
      </svg>
    </button>
  {/if}
</div>
