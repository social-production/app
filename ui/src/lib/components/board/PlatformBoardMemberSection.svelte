<script lang="ts">
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import type { BoardMember } from '#lib/domain/board.ts';
  import { displayName, type Person } from '#lib/domain/people.ts';

  import { resolve } from '$app/paths';

  /** web's PlatformBoardMemberSection: one titled list of moderators or candidates. */
  interface Props {
    title: string;
    members: BoardMember[];
    mode: 'active' | 'candidate';
    sectionIndex: number;
    people: Record<string, Person>;
    onVote: (member: BoardMember, vote: 1 | -1 | 0) => void;
  }

  let { title, members, mode, sectionIndex, people, onVote }: Props = $props();

  const name = (m: BoardMember) => {
    const p = people[m.agent];
    return p ? displayName(p) : 'Unnamed';
  };

  function rowTone(m: BoardMember): string {
    const meets = m.standing === 'active' || m.standing === 'grace' || m.standing === 'qualifying';
    return mode === 'candidate' || !meets ? 'warning' : 'healthy';
  }

  const voteProgress = (m: BoardMember) => `${m.yes + m.no} of ${m.votesRequired} votes needed`;
  const approvalLine = (m: BoardMember) =>
    `66% required · ${m.weeklyActive} active users this week`;

  function statusLabel(m: BoardMember): string {
    if (m.standing === 'active') return 'Standing confirmed';
    if (m.standing === 'grace') return 'Grace period';
    if (m.standing === 'qualifying') return 'Qualifying';
    return 'Below threshold';
  }

  function helperLine(m: BoardMember): string {
    const remaining = Math.max(0, m.votesRequired - (m.yes + m.no));
    const s = remaining === 1 ? '' : 's';
    if (m.standing === 'grace') return 'Grace period — more votes needed before standing drops';
    if (m.standing === 'qualifying') {
      return remaining > 0
        ? `Needs ${remaining} more vote${s} to become moderator`
        : 'Ready to become moderator';
    }
    if (m.standing === 'active') return 'Standing confirmed';
    if (mode === 'candidate') {
      return remaining > 0 ? `Needs ${remaining} more vote${s} to qualify` : statusLabel(m);
    }
    return statusLabel(m);
  }

  const vote = (m: BoardMember, value: 1 | -1) => onVote(m, m.myVote === value ? 0 : value);
</script>

{#if members.length > 0}
  <section class="member-section" class:follows-section={sectionIndex > 0}>
    <div class="section-label">{title}</div>
    {#each members as m (m.agent)}
      {@const href = resolve('/profile/[agent]', { agent: m.agent })}
      <div class={`person-row ${rowTone(m)}`}>
        <a class="person-avatar" {href}>
          <AvatarBadge
            size="sm"
            name={people[m.agent]?.profile?.name ?? ''}
            avatar={people[m.agent]?.profile?.avatar ?? null}
          />
        </a>
        <div class="person-body">
          <a class="person-name" {href}><strong>{name(m)}</strong></a>
          <p class="status-copy">{helperLine(m)}</p>
          <div class="person-footer">
            <div class="person-meta">
              <span class="requirement-value">{voteProgress(m)}</span>
              <span class="requirement-copy">{approvalLine(m)}</span>
            </div>
            <div class="vote-strip" title={`${m.yes} support · ${m.no} oppose`}>
              <button
                aria-label={`Support · ${m.yes}`}
                aria-pressed={m.myVote === 1}
                class:active-support={m.myVote === 1}
                class="vote-button"
                type="button"
                onclick={() => vote(m, 1)}>▲</button
              >
              <span class="signal-percent">
                {m.ratio !== null ? `${(m.ratio * 100).toFixed(0)}%` : '—'}
              </span>
              <button
                aria-label={`Oppose · ${m.no}`}
                aria-pressed={m.myVote === -1}
                class:active-oppose={m.myVote === -1}
                class="vote-button"
                type="button"
                onclick={() => vote(m, -1)}>▼</button
              >
            </div>
          </div>
        </div>
      </div>
    {/each}
  </section>
{/if}
