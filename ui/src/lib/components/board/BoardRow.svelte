<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import { boardBar, boardChip, boardLine, type BoardMember } from '#lib/domain/board.ts';
  import { displayName, type Person } from '#lib/domain/people.ts';

  import { resolve } from '$app/paths';

  /**
   * One person on the Moderators sheet (mockup approved 2026-11-19): photo, name, a chip only
   * when it says something the section doesn't (You, Grace, Ready), one line in words with a
   * thin bar under it, and a joined ↑ | ↓ vote. Your own row has no vote (you can't vote on
   * yourself).
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
  const chip = $derived(boardChip(m));
  const bar = $derived(boardBar(m, quorum));
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
      {#if you}<span class="board-chip you-chip">{t('You')}</span>{/if}
      {#if chip}<span class="board-chip" class:warn={m.standing === 'grace'}>{chip}</span>{/if}
    </div>
    <p class="board-line">{boardLine(m, quorum)}</p>
    {#if bar !== null}
      <span class="board-bar" aria-hidden="true"><i style:width="{bar}%"></i></span>
    {/if}
  </div>
  {#if !you}
    <div class="board-vote" title={t('{yes} yes · {no} no', { yes: m.yes, no: m.no })}>
      <button
        type="button"
        aria-label={t('Yes to {name} · {n}', { name, n: m.yes })}
        aria-pressed={m.myVote === 1}
        class:mine={m.myVote === 1}
        onclick={() => vote(1)}
      >
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 19V5M6 11l6-6 6 6" /></svg>
      </button>
      <button
        type="button"
        aria-label={t('No to {name} · {n}', { name, n: m.no })}
        aria-pressed={m.myVote === -1}
        class:mine={m.myVote === -1}
        onclick={() => vote(-1)}
      >
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M6 13l6 6 6-6" /></svg>
      </button>
    </div>
  {/if}
</div>
