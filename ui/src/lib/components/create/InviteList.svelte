<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import SuggestionList from '#lib/components/messages/SuggestionList.svelte';
  import type { Contact } from '#lib/domain/messages.ts';

  /**
   * A private event's guests (Polish Pass mockups, 2026-11-16): one search adds people, each
   * invited person is a row, and an "Organiser" pill on the row gives them organizer authority
   * (web's second people field). An organizer is always invited too.
   */
  interface Props {
    invitees: Contact[];
    organizers: Contact[];
    search: (query: string) => Promise<Contact[]>;
    onInvite: (contact: Contact) => void;
    onRemove: (agent: string) => void;
    onOrganizer: (contact: Contact, on: boolean) => void;
  }

  let { invitees, organizers, search, onInvite, onRemove, onOrganizer }: Props = $props();

  let query = $state('');
  let found = $state<Contact[]>([]);
  let searchId = 0;

  const items = $derived(found.filter((c) => !invitees.some((s) => s.agent === c.agent)));
  const isOrganizer = (agent: string) => organizers.some((o) => o.agent === agent);
  const initials = (name: string) => name.slice(0, 2).toUpperCase();

  $effect(() => {
    const q = query.trim();
    const id = ++searchId;
    if (!q) {
      found = [];
      return;
    }
    search(q)
      .then((people) => id === searchId && (found = people))
      .catch(() => id === searchId && (found = []));
  });

  function add(contact: Contact): void {
    onInvite(contact);
    query = '';
  }
</script>

<div class="invite-list">
  <label class="tag-search">
    <svg aria-hidden="true" viewBox="0 0 24 24"
      ><circle cx="11" cy="11" r="7" /><path d="m20 20-3.5-3.5" /></svg
    >
    <input bind:value={query} placeholder={t('Add people')} aria-label={t('Invite people')} />
  </label>
  <SuggestionList {items} {query} emptyCopy={t('No matching people.')} onSelect={add} />

  {#if invitees.length > 0}
    <ul class="invite-rows" aria-label={t('Invited')}>
      {#each invitees as person (person.agent)}
        {@const on = isOrganizer(person.agent)}
        <li class="invite-row">
          <span class="invite-avatar" aria-hidden="true">{initials(person.name)}</span>
          <span class="invite-copy">
            <strong>{person.name}</strong>
            <small>{on ? t('Organiser') : t('Invited')}</small>
          </span>
          <button
            type="button"
            class="invite-organiser"
            class:on
            aria-pressed={on}
            onclick={() => onOrganizer(person, !on)}>{t('Organiser')}</button
          >
          <button
            type="button"
            class="invite-remove"
            aria-label={`Remove ${person.name}`}
            onclick={() => onRemove(person.agent)}>×</button
          >
        </li>
      {/each}
    </ul>
  {/if}
  <p class="tag-hint">{t('Organisers can invite people and plan activities with you.')}</p>
</div>
