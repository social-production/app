<script lang="ts">
  import { resolve } from '$app/paths';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import { standingLabel } from '#lib/components/people/standing.ts';
  import { displayName, type Person } from '#lib/domain/people.ts';

  /** One person in a list (web's profile people-list row), linking to their profile. */
  interface Props {
    person: Person;
    /** Devices this person uses; shown when more than one. */
    devices?: number;
  }

  let { person, devices = 1 }: Props = $props();
</script>

<a
  class="person-row"
  href={resolve('/profile/[agent]', { agent: person.agent })}
  data-agent={person.agent}
>
  <span class="person-link">
    <span class="person-head">
      <AvatarBadge
        size="sm"
        name={person.profile?.name ?? ''}
        avatar={person.profile?.avatar ?? null}
      />
      <strong>{displayName(person)}</strong>
      <span class="standing-chip" class:paused={person.standing.level === 'Paused'}
        >{standingLabel(person)}</span
      >
    </span>
    <span class="person-meta">
      {#if person.standing.level === 'New'}<span>{person.standing.vouchers.length}/2 vouches</span
        >{/if}
      {#if devices > 1}<span>{devices} devices</span>{/if}
      {#if person.profile?.home_area}<span>{person.profile.home_area}</span>{/if}
    </span>
  </span>
</a>

<style>
  .person-head {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
    min-width: 0;
  }
  .person-head strong {
    overflow-wrap: anywhere;
  }
  .person-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 12px;
    color: var(--text-soft);
    font-size: 13px;
  }
  .person-meta:empty {
    display: none;
  }
</style>
