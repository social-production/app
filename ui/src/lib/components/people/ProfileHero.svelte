<script lang="ts">
  import type { Snippet } from 'svelte';

  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import { displayName, type Person } from '#lib/domain/people.ts';

  import { standingHint, standingLabel } from './standing.ts';

  /** The header of web's ProfilePage (classes styled by styles/profile.css). */
  interface Props {
    person: Person;
    devices?: number;
    actions?: Snippet;
  }

  let { person, devices = 1, actions }: Props = $props();

  import '#lib/components/styles/profile.css';
</script>

<section class="hero-section">
  <div class="hero-topline">
    <div class="hero-main">
      <div class="hero-identity">
        <AvatarBadge
          size="md"
          name={person.profile?.name ?? ''}
          avatar={person.profile?.avatar ?? null}
        />
        <div class="hero-copy">
          <div class="name-row">
            <h1>{displayName(person)}</h1>
            <span
              class="standing-chip"
              class:paused={person.standing.level === 'Paused'}
              title={standingHint(person)}>{standingLabel(person)}</span
            >
          </div>
          {#if person.profile?.bio}
            <p class="profile-bio">{person.profile.bio}</p>
          {/if}
          <div class="count-row">
            {#if person.profile?.home_area}<span>{person.profile.home_area}</span>{/if}
            {#if devices > 1}<span><strong>{devices}</strong> devices</span>{/if}
            {#if person.standing.level === 'New'}
              <span><strong>{person.standing.vouchers.length}</strong>/2 vouches</span>
            {/if}
          </div>
        </div>
      </div>
      {@render actions?.()}
    </div>
  </div>
</section>
