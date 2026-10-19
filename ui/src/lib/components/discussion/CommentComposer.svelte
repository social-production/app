<script lang="ts">
  import { tick } from 'svelte';

  import '#lib/components/styles/discussion.css';

  /**
   * web's CommentComposer: a growing textarea (up to 240px) with a send button that appears
   * once there is text. Enter sends, Shift+Enter adds a line. @mentions arrive with
   * notifications (M4d).
   */
  interface Props {
    value: string;
    placeholder?: string;
    submitLabel?: string;
    maxLength?: number;
    onSubmit: () => void;
  }

  let {
    value = $bindable(),
    placeholder = 'Write a comment...',
    submitLabel = 'Post comment',
    maxLength,
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

  function keydown(event: KeyboardEvent): void {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      submit();
    }
  }

  function submit(): void {
    if (!value.trim()) return;
    onSubmit();
  }

  /** The parent clears the text after sending; shrink back with it. */
  export async function resetHeight(): Promise<void> {
    await tick();
    resize();
  }

  $effect(() => {
    resize();
  });
</script>

<div class="composer-field">
  <textarea
    bind:this={field}
    bind:value
    class="composer-input"
    class:scrollable
    {placeholder}
    maxlength={maxLength}
    rows="1"
    oninput={resize}
    onkeydown={keydown}></textarea>
  {#if canSubmit}
    <button
      aria-label={submitLabel}
      title={submitLabel}
      class="composer-send"
      type="button"
      onclick={submit}
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
