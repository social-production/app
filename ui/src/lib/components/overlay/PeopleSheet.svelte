<script lang="ts" module>
  import type { Person } from '#lib/domain/people.ts';

  /** A row's button (web's actionLabel/actionKind/actionTone/actionDisabled). */
  export interface PeopleAction {
    label: string;
    danger?: boolean;
    disabled?: boolean;
    run: () => void;
  }

  /** web's PeopleSheetSection: a titled group of people, each with badges and an action. */
  export interface PeopleSection {
    title?: string;
    description?: string;
    emptyCopy: string;
    people: Person[];
    /** Badges after the standing badge (web: Creator, Organizer). */
    badges?: (person: Person) => string[];
    action?: (person: Person) => PeopleAction | undefined;
  }
</script>

<script lang="ts">
  import type { Snippet } from 'svelte';

  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import { standingLabel } from '#lib/components/people/standing.ts';
  import { displayName } from '#lib/domain/people.ts';

  import { resolve } from '$app/paths';

  import OverlaySheet from './OverlaySheet.svelte';

  /** web's PeopleSheet: searchable people in one or more sections, each linking to a profile. */
  interface Props {
    open: boolean;
    title: string;
    description?: string | undefined;
    emptyCopy?: string;
    people?: Person[];
    sections?: PeopleSection[] | null;
    actions?: Snippet | undefined;
    onClose: () => void;
  }

  let {
    open,
    title,
    description,
    emptyCopy = 'No people yet.',
    people = [],
    sections = null,
    actions,
    onClose,
  }: Props = $props();

  let query = $state('');
  const needle = $derived(query.trim().toLowerCase());
  const shown = $derived(
    (sections ?? [{ emptyCopy, people }]).map((section) => ({
      ...section,
      people: section.people
        .filter((p) => !needle || displayName(p).toLowerCase().includes(needle))
        .sort((a, b) =>
          displayName(a).localeCompare(displayName(b), undefined, { sensitivity: 'base' }),
        ),
    })),
  );

  $effect(() => {
    if (!open) query = '';
  });
</script>

<OverlaySheet {open} {title} rootClass="people-sheet" headerActions={actions} {onClose}>
  {#snippet subtitle()}{#if description}<p class="sheet-description">
        {description}
      </p>{/if}{/snippet}
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
  {#each shown as section, index (index)}
    {#if section.title || section.description}
      <div class="section-copy">
        {#if section.title}<h3>{section.title}</h3>{/if}
        {#if section.description}<p>{section.description}</p>{/if}
      </div>
    {/if}
    <div class="people-list">
      {#each section.people as person (person.agent)}
        {@const action = section.action?.(person)}
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
              <span class="badge-row">
                {#each [standingLabel(person), ...(section.badges?.(person) ?? [])] as badge (badge)}
                  <span class="badge">{badge}</span>
                {/each}
              </span>
            </span>
          </a>
          {#if action}
            <button
              class:danger={action.danger}
              class="row-action"
              disabled={action.disabled}
              type="button"
              onclick={action.run}
            >
              {action.label}
            </button>
          {/if}
        </div>
      {:else}
        <p class="empty-row">{needle ? 'No matches.' : section.emptyCopy}</p>
      {/each}
    </div>
    {#if index < shown.length - 1}<div class="section-divider"></div>{/if}
  {/each}
</OverlaySheet>
