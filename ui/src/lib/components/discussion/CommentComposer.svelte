<script lang="ts">
  import { tick } from 'svelte';

  /**
   * web's CommentComposer: a growing textarea (Enter sends, Shift+Enter breaks the line) with a
   * send button once there is text; signed out it is a button asking you to sign in. web's
   * @mention menu comes later (ADR 0070).
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
  }

  let {
    value = $bindable(''),
    placeholder = 'Write a comment...',
    submitLabel = 'Post comment',
    signInMessage = 'Sign in to comment.',
    signedIn,
    onSignIn,
    onSubmit,
  }: Props = $props();

  const MAX_HEIGHT = 240;
  let field = $state<HTMLTextAreaElement>();
  let scrollable = $state(false);
  const canSubmit = $derived(value.trim().length > 0);

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
    onSubmit();
    await tick();
    resize();
  }

  function keydown(event: KeyboardEvent): void {
    if (event.key === 'Enter' && !event.shiftKey) {
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
  {#if signedIn}
    <textarea
      bind:this={field}
      bind:value
      class="composer-input"
      class:scrollable
      {placeholder}
      aria-label={placeholder}
      rows="1"
      oninput={resize}
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
