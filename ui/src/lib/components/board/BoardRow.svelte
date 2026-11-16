<script lang="ts">
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import { boardChip, boardLine, boardProgress, type BoardMember } from '#lib/domain/board.ts';
  import { displayName, type Person } from '#lib/domain/people.ts';

  import { resolve } from '$app/paths';

  /**
   * One person on the Moderators sheet (Platform and Moderators mockups §2): photo, name, a
   * state chip, a bar towards the votes a seat needs (candidates), one line, and the feed's
   * vote control. Your own row says "You" and has no vote buttons (you can't vote on yourself).
   */
  interface Props {
    member: BoardMember;
    person: Person | undefined;
    quorum: number;
    you: boolean;
    onVote: (member: BoardMember, vote: 1 | -1 | 0) => void;
  }

  let { member: m, person, quorum, you, onVote }: Props = $props();

  const name = $derived(person ? displayName(person) : 'Unnamed');
  const pct = $derived(m.ratio === null ? '—' : `${Math.round(m.ratio * 100)}%`);
  const tone = $derived(m.standing === 'grace' ? 'warn' : m.moderator ? 'seated' : 'candidate');
  const vote = (value: 1 | -1) => onVote(m, m.myVote === value ? 0 : value);
</script>

<div class="board-row" class:you>
  <a
    class="board-face"
    href={resolve('/profile/[agent]', { agent: m.agent })}
    aria-hidden="true"
    tabindex="-1"
  >
    <AvatarBadge
      size="sm"
      name={person?.profile?.name ?? ''}
      avatar={person?.profile?.avatar ?? null}
    />
  </a>
  <div class="board-copy">
    <div class="board-name">
      <a href={resolve('/profile/[agent]', { agent: m.agent })}>{name}</a>
      {#if you}<span class="board-chip you-chip">You</span>{/if}
      <span class="board-chip {tone}">{boardChip(m)}</span>
    </div>
    {#if !m.moderator}
      <span class="board-bar" aria-hidden="true"
        ><i style:width="{boardProgress(m, quorum)}%"></i></span
      >
    {/if}
    <p class="board-line">{boardLine(m, quorum)}</p>
  </div>
  {#if !you}
    <div class="board-vote" title={`${m.yes} yes · ${m.no} no`}>
      <button
        type="button"
        aria-label={`Yes to ${name} · ${m.yes}`}
        aria-pressed={m.myVote === 1}
        class:mine={m.myVote === 1}
        onclick={() => vote(1)}
      >
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 19V5M6 11l6-6 6 6" /></svg>
      </button>
      <span>{pct}</span>
      <button
        type="button"
        aria-label={`No to ${name} · ${m.no}`}
        aria-pressed={m.myVote === -1}
        class:mine={m.myVote === -1}
        onclick={() => vote(-1)}
      >
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M6 13l6 6 6-6" /></svg>
      </button>
    </div>
  {/if}
</div>
