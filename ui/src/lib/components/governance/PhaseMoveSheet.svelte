<script lang="ts" module>
  export interface PhaseMove {
    id: string;
    /** "Advance to Activity", "Return to Proposal", "Close project". */
    label: string;
    /** What it does, or why it is not open yet. */
    detail: string;
    disabled: boolean;
    danger?: boolean;
    /** A return's target phase; none for the move on. */
    to?: string;
    /** Convert: close and continue as a new proposal (ADR 0106). */
    convert?: boolean;
  }
</script>

<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';

  import '#lib/components/styles/governance.css';

  /**
   * Every phase move on one sheet (Plan, Phase and Feedback Screens, 4): the choices with
   * what each does (one not open yet is greyed with the reason), the reason members see with
   * the vote, and the button.
   */
  interface Props {
    open: boolean;
    moves: PhaseMove[];
    pending: boolean;
    message: string | undefined;
    onSubmit: (move: PhaseMove, reason: string) => void;
    onClose: () => void;
  }

  let { open, moves, pending, message, onSubmit, onClose }: Props = $props();

  let chosen = $state('');
  let reason = $state('');
  let local = $state('');

  $effect(() => {
    if (!open) return;
    chosen = moves.find((m) => !m.disabled)?.id ?? '';
    reason = '';
    local = '';
  });

  const move = $derived(moves.find((m) => m.id === chosen && !m.disabled));
  const alert = $derived(local || message);

  function submit(): void {
    if (!move) return;
    if (!reason.trim()) {
      local = 'Add a reason before starting the vote.';
      return;
    }
    local = '';
    onSubmit(move, reason.trim());
  }
</script>

<OverlaySheet
  {open}
  title={t('Where should this go next?')}
  labelledById="phase-move-sheet"
  rootClass="phase-move-sheet"
  {onClose}
>
  <div class="phase-moves" role="radiogroup" aria-label={t('Phase moves')}>
    {#each moves as option (option.id)}
      <button
        role="radio"
        aria-checked={chosen === option.id}
        class="phase-move"
        class:on={chosen === option.id}
        class:danger={option.danger}
        disabled={option.disabled}
        type="button"
        onclick={() => (chosen = option.id)}
      >
        <span class="radio" aria-hidden="true"></span>
        <span class="phase-move-copy">
          <strong>{option.label}</strong>
          <small>{option.detail}</small>
        </span>
      </button>
    {/each}
  </div>
  <label class="phase-move-reason">
    <span>{t('Reason, shown to members with the vote')}</span>
    <textarea
      bind:value={reason}
      rows="3"
      maxlength="2000"
      placeholder={t('State why this phase change should happen now.')}></textarea>
  </label>
  {#if alert}<p class="phase-move-alert" role="alert">{alert}</p>{/if}
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-submit" disabled={pending || !move} type="button" onclick={submit}
        >{pending ? t('Working...') : t('Start the vote')}</button
      >
    </div>
  {/snippet}
</OverlaySheet>
