<script lang="ts">
  import { MESSAGE_MAX, type ChatLine } from '#lib/domain/chat.ts';

  /** web LiveChatPanel's composer card: the reply bar, the growing textarea and Send. */
  interface Props {
    placeholder: string;
    submitLabel: string;
    replyTarget: ChatLine | null;
    paused: boolean;
    error: string;
    /** Resolves once sent; throws to keep the draft. */
    onSubmit: (body: string) => Promise<void>;
    onCancelReply: () => void;
    onFocus: () => void;
  }

  let p: Props = $props();

  const COMPOSER_MAX_HEIGHT = 132;

  let draft = $state('');
  let pending = $state(false);
  let field = $state<HTMLTextAreaElement>();
  const canSend = $derived(Boolean(draft.trim()));

  $effect(() => {
    if (field && draft === '') field.style.height = '';
  });

  function grow(): void {
    if (!field) return;
    field.style.height = 'auto';
    field.style.height = `${Math.min(field.scrollHeight, COMPOSER_MAX_HEIGHT)}px`;
  }

  async function submit(): Promise<void> {
    const body = draft.trim();
    if (!body || pending) return;
    draft = '';
    pending = true;
    try {
      await p.onSubmit(body);
    } catch {
      draft = body;
    } finally {
      pending = false;
    }
  }

  function keydown(event: KeyboardEvent): void {
    if (event.key !== 'Enter' || event.shiftKey) return;
    // On phones Enter adds a line, as in other messaging apps; the send button sends.
    if (window.matchMedia('(pointer: coarse)').matches) return;
    event.preventDefault();
    void submit();
  }
</script>

<div class="composer-card">
  {#if p.replyTarget}
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
      <button
        type="button"
        aria-label="Cancel reply"
        title="Cancel reply"
        onclick={p.onCancelReply}
      >
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
    </div>
  {/if}
  {#if p.error}
    <p class="attach-error" role="alert">{p.error}</p>
  {/if}
  {#if p.paused}
    <p class="composer-paused" role="status">
      This account is read-only: a community review found it's a bot.
    </p>
  {:else}
    <div class="composer-row">
      <div class="composer-input-shell">
        <textarea
          bind:this={field}
          bind:value={draft}
          maxlength={MESSAGE_MAX}
          onfocus={p.onFocus}
          oninput={grow}
          onkeydown={keydown}
          placeholder={p.placeholder}
          aria-label={p.placeholder}
          rows="1"></textarea>
      </div>
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
    </div>
  {/if}
</div>
