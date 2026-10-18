<script lang="ts">
  import type { Snippet } from 'svelte';

  import AvatarImage from '#lib/components/AvatarImage.svelte';
  import StatusPill from '#lib/components/StatusPill.svelte';
  import { displayName, type Person } from '#lib/domain/people.ts';

  interface Props {
    person: Person;
    actions?: Snippet;
  }

  let { person, actions }: Props = $props();

  const standing = $derived(
    person.standing.founder
      ? { tone: 'ok' as const, label: 'Founder' }
      : person.standing.level === 'Full'
        ? { tone: 'ok' as const, label: 'Full member' }
        : person.standing.level === 'Paused'
          ? { tone: 'error' as const, label: 'Paused' }
          : {
              tone: 'pending' as const,
              label: `New · ${person.standing.vouchers.length}/2 vouches`,
            },
  );
</script>

<div class="chip">
  <AvatarImage avatar={person.profile?.avatar ?? null} name={person.profile?.name ?? ''} />
  <div class="text">
    <strong>{displayName(person)}</strong>
    {#if person.profile?.home_area}<span class="muted">{person.profile.home_area}</span>{/if}
  </div>
  <StatusPill tone={standing.tone} label={standing.label} />
  {@render actions?.()}
</div>

<style>
  .chip {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }
  .text {
    display: grid;
    flex: 1;
    min-width: 140px;
  }
</style>
