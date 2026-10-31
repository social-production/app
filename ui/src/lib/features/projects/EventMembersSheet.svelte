<script lang="ts">
  import PeopleSheet, { type PeopleSection } from '#lib/components/overlay/PeopleSheet.svelte';
  import ShareUserMenu from '#lib/components/overlay/ShareUserMenu.svelte';
  import type { Contact } from '#lib/domain/messages.ts';
  import type { Person } from '#lib/domain/people.ts';
  import { explain } from '#lib/services/errors.ts';
  import { addOrganizer, sendEventInvite } from '#lib/services/events.ts';
  import { contacts } from '#lib/services/messages.ts';
  import { getPerson, myAgent } from '#lib/services/people.ts';

  import type { DetailState } from './detailState.svelte.ts';

  /**
   * web's EventMembersPanel: a public event lists everyone who joined; a private one splits
   * Organizers from Members, and organizers promote members and invite people (ADR 0061).
   * web's "Remove organizer" waits on revocable grants (an integrity change).
   */
  interface Props {
    detail: DetailState;
    open: boolean;
    onClose: () => void;
  }

  let { detail, open, onClose }: Props = $props();

  const project = $derived(detail.project);
  const isPrivate = $derived(!!project && project.audience !== 'public');
  const organized = $derived(project?.governance === 'organizer_controlled');
  const canManage = $derived(isPrivate && !!project?.amOrganizer);
  /** Grants written here but not yet in the network read (a write lags its own reads). */
  let promoted = $state<string[]>([]);
  let pending = $state<string | null>(null);
  let organizerPeople = $state<Person[]>([]);
  let error = $state('');

  const organizerKeys = $derived([...new Set([...(project?.organizers ?? []), ...promoted])]);

  $effect(() => {
    const keys = organizerKeys;
    if (!open || !isPrivate) return;
    const known = new Map(detail.members.map((m) => [m.agent, m]));
    void Promise.all(keys.map((k) => known.get(k) ?? getPerson(k))).then(
      (people) => (organizerPeople = people),
    );
  });

  const primaryCopy = $derived(
    organized
      ? 'Organizers can invite people, promote organizers, create activities, and manage event decisions directly.'
      : 'Organizers can invite people. Members can still propose values, plans, activities, and lifecycle decisions.',
  );
  const secondaryCopy = $derived(
    organized
      ? 'Members can attend and sign up for activity roles. Promote a member to organizer when they should help manage the event.'
      : 'Members can propose, vote, attend, and sign up for roles. Only organizers can invite people.',
  );
  const description = $derived(
    !isPrivate
      ? 'Public event members can propose and vote on update and detail edit decisions.'
      : organized
        ? 'Invite people and promote organizers from this panel. Members join to attend and sign up for roles.'
        : 'Invite people and promote organizers from this panel. Members can still propose and vote inside this private audience.',
  );

  const sections = $derived<PeopleSection[]>(
    isPrivate
      ? [
          {
            title: 'Organizers',
            description: primaryCopy,
            emptyCopy: 'No organizers yet.',
            people: organizerPeople,
            badges: (p) =>
              p.agent === project?.creator ? ['Creator', 'Organizer'] : ['Organizer'],
          },
          {
            title: 'Members',
            description: secondaryCopy,
            emptyCopy: 'No additional members have joined yet.',
            people: detail.members.filter((m) => !organizerKeys.includes(m.agent)),
            action: (p) =>
              canManage
                ? {
                    label: 'Promote to organizer',
                    disabled: pending === p.agent,
                    run: () => void promote(p.agent),
                  }
                : undefined,
          },
        ]
      : [
          {
            emptyCopy: 'No one has joined this event yet.',
            people: detail.members,
            badges: (p) => (p.agent === project?.creator ? ['Creator'] : []),
          },
        ],
  );

  async function promote(agent: string): Promise<void> {
    pending = agent;
    error = '';
    try {
      await addOrganizer(detail.id, agent);
      promoted = [...promoted, agent];
      void detail.load();
    } catch (e) {
      error = explain(e);
    } finally {
      pending = null;
    }
  }

  let people: Promise<Contact[]> | undefined;
  /** People to invite: not yourself, not already in the event. */
  async function search(query: string): Promise<Contact[]> {
    people ??= Promise.all([contacts(), myAgent()]).then(([all, me]) =>
      all.filter((c) => c.agent !== me),
    );
    const inside = new Set([...detail.members.map((m) => m.agent), ...organizerKeys]);
    const needle = query.trim().toLowerCase();
    return (await people)
      .filter((c) => !inside.has(c.agent) && c.name.toLowerCase().includes(needle))
      .slice(0, 8);
  }

  async function invite(contact: Contact): Promise<string | null> {
    try {
      await sendEventInvite(contact.agent, detail.id);
      return null;
    } catch (e) {
      return explain(e);
    }
  }
</script>

<PeopleSheet
  {open}
  title="Members"
  description={error || description}
  {sections}
  actions={canManage ? inviteMenu : undefined}
  {onClose}
/>

{#snippet inviteMenu()}
  <ShareUserMenu
    menuTitle="Invite to event"
    placeholder="Search people"
    submitLabel="Invite"
    searchContacts={search}
    submitShare={invite}
  />
{/snippet}
