<script lang="ts">
  import type { Stance } from '#lib/domain/projects.ts';

  /**
   * web's VoteStrip in signals mode: ▲ support · favorability % · ▼ oppose. Clicking your
   * current stance withdraws it. Counts update at once and roll back if the write fails.
   */
  interface Props {
    support: number;
    oppose: number;
    mine: Stance | null;
    disabled?: boolean;
    labeled?: boolean;
    corner?: boolean;
    onSignal: (next: Stance | null) => Promise<void>;
  }

  let {
    support,
    oppose,
    mine,
    disabled = false,
    labeled = false,
    corner = false,
    onSignal,
  }: Props = $props();

  let local = $state<{ support: number; oppose: number; mine: Stance | null } | null>(null);
  const shown = $derived(local ?? { support, oppose, mine });
  const total = $derived(shown.support + shown.oppose);
  const percent = $derived(total > 0 ? `${Math.round((shown.support / total) * 100)}%` : '—');
  const tooltip = $derived(
    total > 0
      ? `${shown.support} support · ${shown.oppose} oppose · ${percent} support`
      : `${shown.support} support · ${shown.oppose} oppose · no signals yet`,
  );

  async function click(stance: Stance, event: MouseEvent): Promise<void> {
    event.stopPropagation();
    if (disabled) return;
    const before = shown;
    const next = before.mine === stance ? null : stance;
    const count = (s: Stance) =>
      (s === 'Demand' ? before.support : before.oppose) -
      Number(before.mine === s) +
      Number(next === s);
    local = { support: count('Demand'), oppose: count('Oppose'), mine: next };
    try {
      await onSignal(next);
    } catch {
      local = before;
    }
  }

  import '#lib/components/styles/cards.css';
</script>

<div class="vote-strip signal-strip" class:disabled class:labeled class:corner title={tooltip}>
  <button
    aria-label={`Support · ${shown.support}`}
    aria-pressed={shown.mine === 'Demand'}
    class:active-support={shown.mine === 'Demand'}
    class="signal-button vote-button"
    {disabled}
    type="button"
    onclick={(e) => click('Demand', e)}
  >
    ▲
    {#if labeled}<span class="signal-label">Support</span><span class="signal-count"
        >{shown.support}</span
      >{/if}
  </button>
  <span class="signal-percent" aria-label={tooltip}>{percent}</span>
  <button
    aria-label={`Oppose · ${shown.oppose}`}
    aria-pressed={shown.mine === 'Oppose'}
    class:active-oppose={shown.mine === 'Oppose'}
    class="signal-button vote-button"
    {disabled}
    type="button"
    onclick={(e) => click('Oppose', e)}
  >
    {#if labeled}<span class="signal-label">Oppose</span><span class="signal-count"
        >{shown.oppose}</span
      >{/if}
    ▼
  </button>
</div>
