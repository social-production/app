<script lang="ts">
  import type { Stance } from '#lib/domain/projects.ts';

  import '#lib/components/styles/cards.css';

  /**
   * web's VoteStrip in votes mode: ▲ count ▼. A help request's votes are its Support/Oppose
   * signals (ADR 0067), so the count is support minus opposition; clicking your vote again
   * withdraws it. The count moves at once and rolls back if the write fails.
   */
  interface Props {
    support: number;
    oppose: number;
    mine: Stance | null;
    docked?: boolean;
    corner?: boolean;
    /** Feed rows (ADR 0096): no box, drawn arrows, styled by styles/ux/feed.css. */
    bare?: boolean;
    onSignal: (next: Stance | null) => Promise<void>;
  }

  let {
    support,
    oppose,
    mine,
    docked = false,
    corner = false,
    bare = false,
    onSignal,
  }: Props = $props();

  let local = $state<{ count: number; mine: Stance | null } | null>(null);
  const shown = $derived(local ?? { count: support - oppose, mine });

  async function click(stance: Stance, event: MouseEvent): Promise<void> {
    event.stopPropagation();
    const before = shown;
    const next = before.mine === stance ? null : stance;
    const weight = (s: Stance | null) => (s === 'Demand' ? 1 : s === 'Oppose' ? -1 : 0);
    local = { count: before.count - weight(before.mine) + weight(next), mine: next };
    try {
      await onSignal(next);
    } catch {
      local = before;
    }
  }
</script>

<div class={bare ? 'feed-vote' : 'vote-strip'} class:docked class:corner={corner && !bare}>
  <button
    aria-label="Vote up"
    aria-pressed={shown.mine === 'Demand'}
    class:active-up={shown.mine === 'Demand'}
    class="vote-button"
    type="button"
    onclick={(e) => void click('Demand', e)}
    >{#if bare}<svg aria-hidden="true" viewBox="0 0 24 24" class="feed-vote-icon"
        ><path
          d="M12 5.5 5 13h4v5.5h6V13h4L12 5.5Z"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linejoin="round"
        /></svg
      >{:else}▲{/if}</button
  >
  <span
    class:active-up={shown.mine === 'Demand'}
    class:active-down={shown.mine === 'Oppose'}
    class="vote-count">{shown.count}</span
  >
  <button
    aria-label="Vote down"
    aria-pressed={shown.mine === 'Oppose'}
    class:active-down={shown.mine === 'Oppose'}
    class="vote-button"
    type="button"
    onclick={(e) => void click('Oppose', e)}
    >{#if bare}<svg aria-hidden="true" viewBox="0 0 24 24" class="feed-vote-icon"
        ><path
          d="M12 18.5 5 11h4V5.5h6V11h4l-7 7.5Z"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linejoin="round"
        /></svg
      >{:else}▼{/if}</button
  >
</div>
