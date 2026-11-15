<script lang="ts">
  import { resolve } from '$app/paths';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import { standingLabel } from '#lib/components/people/standing.ts';
  import type { Person } from '#lib/domain/people.ts';

  /** One person in the People list (Remaining Screens, 6): a plain row with a larger face,
   * the name, one quiet line and "View". The key suffix stays on the profile. */
  interface Props {
    person: Person;
    /** Devices this person uses; shown when more than one. */
    devices?: number;
  }

  let { person, devices = 1 }: Props = $props();

  const facts = $derived(
    [
      standingLabel(person),
      person.standing.level === 'New' ? `${person.standing.vouchers.length} of 2 vouches` : '',
      devices > 1 ? `${devices} devices` : '',
      person.profile?.home_area ?? '',
    ].filter(Boolean),
  );
</script>

<a
  class="person-row people-row"
  class:paused={person.standing.level === 'Paused'}
  href={resolve('/profile/[agent]', { agent: person.agent })}
  data-agent={person.agent}
>
  <AvatarBadge
    size="md"
    name={person.profile?.name ?? ''}
    avatar={person.profile?.avatar ?? null}
  />
  <span class="people-copy">
    <strong>{person.profile?.name ?? 'Unnamed'}</strong>
    <small>{facts.join(' · ')}</small>
  </span>
  <span class="people-view">View</span>
</a>
