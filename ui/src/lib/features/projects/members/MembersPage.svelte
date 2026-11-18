<script lang="ts">
  import { locale, t } from '#lib/i18n/index.ts';
  import { untrack } from 'svelte';

  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import type { Person } from '#lib/domain/people.ts';
  import { placeAndStanding } from '#lib/domain/personLine.ts';
  import type { EventInvite } from '#lib/services/eventInvites.ts';
  import { cachedPerson, myAgent } from '#lib/services/people.ts';

  import DetailShare from '../DetailShare.svelte';
  import type { DetailState } from '../detailState.svelte.ts';
  import MemberRow from './MemberRow.svelte';
  import { OrganizerActions } from './organizerActions.svelte.ts';

  import '#lib/components/styles/ux/members.css';

  /**
   * The member list as a full page (Detail Pages Cleanup §6, v5): search, then Members and,
   * on every event, Invited (sent an invitation, not joined yet; ADR 0111). A private event
   * lists its organisers first. Projects show Members only.
   */
  interface Props {
    detail: DetailState;
    /** Every invitation on the event; `null` for a project (no Invited side). */
    invites: EventInvite[] | null;
    onClose: () => void;
  }

  let { detail, invites, onClose }: Props = $props();

  // The page opens on one detail, so reading `detail` once is intended.
  // svelte-ignore state_referenced_locally
  const actions = new OrganizerActions(detail);
  let side = $state<'members' | 'invited'>('members');
  let query = $state('');
  let me = $state('');
  let people = $state<Record<string, Person>>({});

  const project = $derived(detail.project);
  const isPrivate = $derived(!!project && project.audience !== 'public');
  const canManage = $derived(isPrivate && !!project?.amOrganizer);
  const memberKeys = $derived(new Set(detail.members.map((m) => m.agent)));
  const waiting = $derived((invites ?? []).filter((i) => !memberKeys.has(i.agent)));
  const organizers = $derived(isPrivate ? actions.organizers() : []);
  const needle = $derived(query.trim().toLowerCase());
  const nameOf = (agent: string): string =>
    people[agent]?.profile?.name || detail.names[agent] || 'Unnamed';
  const matches = (agent: string): boolean =>
    !needle || nameOf(agent).toLowerCase().includes(needle);
  // Your row first, then by name.
  const order = (a: string, b: string): number =>
    Number(b === me) - Number(a === me) || nameOf(a).localeCompare(nameOf(b));
  const lead = $derived(organizers.filter(matches).sort(order));
  const rest = $derived(
    detail.members
      .map((m) => m.agent)
      .filter((a) => !organizers.includes(a) && matches(a))
      .sort(order),
  );
  const invited = $derived(waiting.filter((i) => matches(i.agent)));

  $effect(() => {
    void myAgent().then((agent) => (me = agent));
  });

  $effect(() => {
    const known = Object.fromEntries(detail.members.map((m) => [m.agent, m]));
    const missing = [...organizers, ...waiting.flatMap((i) => [i.agent, i.by])].filter(
      (a) => !known[a],
    );
    // `people` is written here, so it is read untracked (or the effect re-runs itself).
    people = { ...untrack(() => people), ...known };
    for (const agent of new Set(missing)) {
      void cachedPerson(agent).then((p) => (people = { ...untrack(() => people), [agent]: p }));
    }
  });

  function invitedLine(i: EventInvite): string {
    const place = people[i.agent]?.profile?.home_area.trim();
    const when = new Date(i.at).toLocaleDateString(locale(), { day: 'numeric', month: 'short' });
    const by = `invited by ${i.by === me ? 'you' : nameOf(i.by)}, ${when}`;
    return place ? `${place} · ${by}` : by;
  }

  function line(agent: string): string {
    const person = people[agent];
    return person ? placeAndStanding(person) : '';
  }

  function manage(agent: string, organizer: boolean) {
    if (!canManage || agent === project?.creator) return [];
    return organizer
      ? [{ label: t('Remove'), icon: 'x', danger: true, run: () => void actions.demote(agent) }]
      : [{ label: t('Make organiser'), icon: 'shield', run: () => void actions.promote(agent) }];
  }
</script>

{#snippet row(agent: string, organizer: boolean)}
  <MemberRow
    {agent}
    name={nameOf(agent)}
    line={line(agent)}
    you={agent === me}
    badge={organizer ? t('Organiser') : ''}
    actions={manage(agent, organizer)}
    busy={actions.pending === agent}
  />
{/snippet}

<div class="members-page" role="dialog" aria-modal="true" aria-label={t('People')}>
  <header class="members-head">
    <button type="button" class="members-back" aria-label={t('Back')} onclick={onClose}>
      <svg aria-hidden="true" viewBox="0 0 24 24"><path d="M14.5 5.5 8 12l6.5 6.5" /></svg>
    </button>
    <h1>{t('People')}</h1>
  </header>
  {#if invites}
    <div class="members-sides" role="tablist" aria-label={t('Members or invited')}>
      <button
        type="button"
        role="tab"
        aria-selected={side === 'members'}
        onclick={() => (side = 'members')}>Members · {detail.members.length}</button
      >
      <button
        type="button"
        role="tab"
        aria-selected={side === 'invited'}
        onclick={() => (side = 'invited')}>Invited · {waiting.length}</button
      >
    </div>
  {/if}
  <label class="members-search">
    <ToolbarIcon name="search" />
    <input
      type="search"
      placeholder={t('Search')}
      aria-label={t('Search people')}
      bind:value={query}
    />
  </label>
  {#if actions.error}<p class="members-note" role="alert">{actions.error}</p>{/if}

  <div class="members-list">
    {#if side === 'invited' && invites}
      <h2>Waiting to join · {waiting.length}</h2>
      {#each invited as i (i.agent)}
        <MemberRow agent={i.agent} name={nameOf(i.agent)} line={invitedLine(i)} />
      {:else}
        <p class="members-note">
          {needle ? t('No matches.') : t('Nobody is waiting: everyone invited has joined.')}
        </p>
      {/each}
    {:else}
      {#if isPrivate}
        <h2>Organisers · {organizers.length}</h2>
        {#each lead as agent (agent)}{@render row(agent, true)}{/each}
        <h2>Members · {detail.members.filter((m) => !organizers.includes(m.agent)).length}</h2>
      {/if}
      {#each rest as agent (agent)}{@render row(agent, false)}{/each}
      {#if rest.length === 0}
        <p class="members-note">{needle ? t('No matches.') : t('Nobody else has joined yet.')}</p>
      {/if}
    {/if}
  </div>

  <div class="members-invite"><DetailShare {detail} label={t('Invite people')} /></div>
</div>
