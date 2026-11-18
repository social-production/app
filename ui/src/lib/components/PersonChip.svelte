<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { resolve } from '$app/paths';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import { standingLabel } from '#lib/components/people/standing.ts';
  import type { Person } from '#lib/domain/people.ts';

  /** One person in the People list (Screen Review §2, owner 2026-11-18): the whole row opens
   * them; the second line is place and standing as in member lists, with the vouch count only
   * part-way there; a chevron instead of a "View" box. The key suffix stays on the profile. */
  interface Props {
    person: Person;
    /** Devices this person uses; shown when more than one. */
    devices?: number;
  }

  let { person, devices = 1 }: Props = $props();

  const facts = $derived(
    [
      person.profile?.home_area ?? '',
      standingLabel(person),
      person.standing.level === 'New' && person.standing.vouchers.length > 0
        ? `${person.standing.vouchers.length} of 2 vouches`
        : '',
      devices > 1 ? `${devices} devices` : '',
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
    <strong>{person.profile?.name ?? t('Unnamed')}</strong>
    <small>{facts.join(' · ')}</small>
  </span>
  <svg class="people-chevron" aria-hidden="true" viewBox="0 0 24 24"><path d="m10 7 5 5-5 5" /></svg
  >
</a>
