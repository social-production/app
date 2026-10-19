<script lang="ts">
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import { standingLabel } from '#lib/components/people/standing.ts';
  import { displayName, type Person } from '#lib/domain/people.ts';

  import { resolve } from '$app/paths';

  import OverlaySheet from './OverlaySheet.svelte';

  /** web's PeopleSheet (one section): searchable people, each linking to their profile. */
  interface Props {
    open: boolean;
    title: string;
    description: string;
    emptyCopy: string;
    people: Person[];
    onClose: () => void;
  }

  let { open, title, description, emptyCopy, people, onClose }: Props = $props();

  let query = $state('');
  const needle = $derived(query.trim().toLowerCase());
  const shown = $derived(
    people
      .filter((p) => !needle || displayName(p).toLowerCase().includes(needle))
      .sort((a, b) =>
        displayName(a).localeCompare(displayName(b), undefined, { sensitivity: 'base' }),
      ),
  );

  $effect(() => {
    if (!open) query = '';
  });
</script>

<OverlaySheet {open} {title} rootClass="people-sheet" {onClose}>
  {#snippet subtitle()}<p class="sheet-description">{description}</p>{/snippet}
  {#snippet toolbar()}
    <div class="sheet-search">
      <label class="sr-only" for="people-sheet-search">Search people</label>
      <input
        id="people-sheet-search"
        bind:value={query}
        placeholder="Search people"
        type="search"
      />
    </div>
  {/snippet}
  <div class="people-list">
    {#each shown as person (person.agent)}
      <div class="person-row">
        <a
          class="person-main"
          href={resolve('/profile/[agent]', { agent: person.agent })}
          onclick={onClose}
        >
          <AvatarBadge
            size="sm"
            name={person.profile?.name ?? ''}
            avatar={person.profile?.avatar ?? null}
          />
          <span class="person-copy">
            <strong>{displayName(person)}</strong>
            <span class="badge-row"><span class="badge">{standingLabel(person)}</span></span>
          </span>
        </a>
      </div>
    {:else}
      <p class="empty-row">{needle ? 'No matches.' : emptyCopy}</p>
    {/each}
  </div>
</OverlaySheet>
