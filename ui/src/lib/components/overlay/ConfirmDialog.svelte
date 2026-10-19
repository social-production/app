<script lang="ts">
  import { portal } from '#lib/components/actions/portal.ts';

  import '#lib/components/styles/confirm.css';

  /** web's ConfirmHost (askConfirm): a small alert dialog with Keep and a confirm button. */
  interface Props {
    open: boolean;
    title: string;
    body?: string;
    confirmLabel?: string;
    danger?: boolean;
    onSettle: (confirmed: boolean) => void;
  }

  let {
    open,
    title,
    body = '',
    confirmLabel = 'Confirm',
    danger = false,
    onSettle,
  }: Props = $props();

  let confirmButton = $state<HTMLButtonElement>();

  $effect(() => {
    if (open) confirmButton?.focus();
  });
</script>

<svelte:window
  onkeydown={(event) => {
    if (open && event.key === 'Escape') onSettle(false);
  }}
/>

{#if open}
  <div class="confirm-root" use:portal>
    <button class="confirm-scrim" type="button" aria-label="Cancel" onclick={() => onSettle(false)}
    ></button>
    <div
      class="confirm-card"
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="confirm-dialog-title"
    >
      <h2 id="confirm-dialog-title">{title}</h2>
      {#if body}
        <p>{body}</p>
      {/if}
      <div class="confirm-actions">
        <button class="keep" type="button" onclick={() => onSettle(false)}>Keep</button>
        <button
          bind:this={confirmButton}
          class="go"
          class:danger
          type="button"
          onclick={() => onSettle(true)}>{confirmLabel}</button
        >
      </div>
    </div>
  </div>
{/if}
