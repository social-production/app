<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import SkeletonRows from '#lib/components/loading/SkeletonRows.svelte';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import GroupsIcon from '#lib/components/icons/GroupsIcon.svelte';
  import type { Contact } from '#lib/domain/messages.ts';
  import PlaceStanding from '#lib/components/people/PlaceStanding.svelte';
  import { clashingNames, pickerName } from '#lib/domain/personLine.ts';
  import { cachedPerson } from '#lib/services/people.ts';

  import NewGroupName from './NewGroupName.svelte';

  /**
   * New message as Telegram does it (owner, 2026-11-14): a full page, not a form. A "New
   * group" row, then everyone as rows; tapping a person opens the chat with them. New group
   * uses the same list: tapping ticks a person and adds a chip, the round arrow goes on to
   * name the group and create it.
   */
  interface Props {
    group: boolean;
    people: () => Promise<Contact[]>;
    onBack: () => void;
    onPerson: (person: Contact) => void;
    onGroup: () => void;
    /** Creates the group; a thrown error is shown and keeps the choices. */
    onCreate: (members: Contact[], title: string) => Promise<void>;
  }

  let { group, people, onBack, onPerson, onGroup, onCreate }: Props = $props();

  let everyone = $state<Contact[] | null>(null);
  let failed = $state(false);
  let query = $state('');
  let chosen = $state<Contact[]>([]);
  let naming = $state(false);

  const clashes = $derived(clashingNames(everyone ?? []));
  const shown = $derived.by(() => {
    const needle = query.trim().toLowerCase();
    return (everyone ?? [])
      .filter((c) => c.name.toLowerCase().includes(needle))
      .sort((a, b) => a.name.localeCompare(b.name));
  });
  const isChosen = (c: Contact): boolean => chosen.some((m) => m.agent === c.agent);

  function load(): void {
    failed = false;
    people().then(
      (list) => (everyone = list),
      () => (failed = true),
    );
  }
  $effect(load);

  // Leaving New group forgets its choices.
  $effect(() => {
    if (!group) {
      chosen = [];
      naming = false;
    }
  });

  function tap(c: Contact): void {
    if (!group) return onPerson(c);
    chosen = isChosen(c) ? chosen.filter((m) => m.agent !== c.agent) : [...chosen, c];
    query = '';
  }
</script>

{#if group && naming}
  <NewGroupName members={chosen} onBack={() => (naming = false)} {onCreate} />
{:else}
  <div class="new-message">
    <header class="new-message-head">
      <button
        class="round-back"
        type="button"
        aria-label={t('Back')}
        title={t('Back')}
        onclick={onBack}
      >
        <svg aria-hidden="true" viewBox="0 0 24 24"><path d="M15 5l-7 7 7 7" /></svg>
      </button>
      <div class="new-message-title">
        <h1>{group ? t('New group') : t('New message')}</h1>
        {#if group}<p>{chosen.length} selected</p>{/if}
      </div>
    </header>

    {#if group && chosen.length > 0}
      <div class="new-message-chips" aria-label={t('Selected people')}>
        {#each chosen as member (member.agent)}
          <button
            class="person-chip"
            type="button"
            aria-label={`Remove ${member.name}`}
            onclick={() => tap(member)}
          >
            <AvatarBadge size="sm" name={member.name} />
            <span>{member.name}</span>
          </button>
        {/each}
      </div>
    {/if}

    <label class="new-message-search">
      <svg aria-hidden="true" viewBox="0 0 24 24">
        <circle cx="11" cy="11" r="6" /><path d="m20 20-4.5-4.5" />
      </svg>
      <input
        bind:value={query}
        type="search"
        aria-label={t('Search people')}
        placeholder={group ? t('Who would you like to add?') : t('Search')}
      />
    </label>

    <div class="new-message-list">
      {#if !group && !query.trim()}
        <button class="person-row group-row" type="button" onclick={onGroup}>
          <span class="group-glyph"><GroupsIcon /></span>
          <span class="person-copy"><strong>{t('New group')}</strong></span>
        </button>
      {/if}
      {#if failed}
        <p class="new-message-note">
          {t('People could not be loaded.')}
          <button type="button" onclick={load}>{t('Try again')}</button>
        </p>
      {:else if everyone === null}
        <SkeletonRows count={4} kind="row" />
      {:else if shown.length === 0}
        <p class="new-message-note">
          {query.trim() ? t('No matches.') : t('Nobody else is here yet.')}
        </p>
      {:else}
        {#each shown as person (person.agent)}
          {@const ticked = group && isChosen(person)}
          <button
            class="person-row"
            type="button"
            aria-pressed={group ? ticked : undefined}
            onclick={() => tap(person)}
          >
            <span class="person-face">
              <AvatarBadge size="md" name={person.name} />
              {#if ticked}
                <span class="person-tick" aria-hidden="true">
                  <svg viewBox="0 0 24 24"><path d="m6 12.5 4 4 8-9" /></svg>
                </span>
              {/if}
            </span>
            <span class="person-copy">
              <strong>{pickerName(person, clashes)}</strong>
              <small><PlaceStanding agent={person.agent} load={cachedPerson} /></small>
            </span>
          </button>
        {/each}
      {/if}
    </div>

    {#if group}
      <button
        class="message-fab"
        type="button"
        aria-label={t('Next')}
        title={t('Next')}
        disabled={chosen.length === 0}
        onclick={() => (naming = true)}
      >
        <svg aria-hidden="true" viewBox="0 0 24 24">
          <path
            d="M5 12h14M13 6l6 6-6 6"
            fill="none"
            stroke="currentColor"
            stroke-width="2.2"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
    {/if}
  </div>
{/if}
