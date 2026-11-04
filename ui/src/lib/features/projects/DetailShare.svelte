<script lang="ts">
  import ShareUserMenu from '#lib/components/overlay/ShareUserMenu.svelte';
  import type { Contact } from '#lib/domain/messages.ts';
  import { isEvent } from '#lib/domain/projects.ts';
  import { explain } from '#lib/services/errors.ts';
  import { sendEventInvite } from '#lib/services/events.ts';
  import { contacts } from '#lib/services/messages.ts';
  import { myAgent } from '#lib/services/people.ts';
  import { sharePrefill, shareSpace } from '#lib/services/shares.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  import type { DetailState } from './detailState.svelte.ts';

  /**
   * web's "Share +" on ProjectOverviewHeader / EventOverviewHeader. Projects: anyone signed in
   * sees it, members send (web-backend refuses others). Public events: members. Private
   * events: organizers, as "Invite to event" (an invite to accept, no Create post).
   */
  let { detail }: { detail: DetailState } = $props();

  const project = $derived(detail.project);
  const event = $derived(isEvent(project?.charter?.mode));
  const isPrivate = $derived(event && project?.audience !== 'public');
  const signedIn = $derived(connection.current.status === 'connected');
  const shown = $derived(
    signedIn && !!project && (!event || (isPrivate ? !!project.amOrganizer : project.amMember)),
  );
  const url = $derived(`${location.origin}${location.pathname}`);
  const title = $derived(project?.charter?.title ?? '');

  let people: Promise<Contact[]> | null = null;

  async function search(query: string): Promise<Contact[]> {
    people ??= Promise.all([contacts(), myAgent()]).then(([all, me]) =>
      all.filter((c) => c.agent !== me),
    );
    const needle = query.trim().toLowerCase();
    return (await people).filter((c) => c.name.toLowerCase().includes(needle)).slice(0, 8);
  }

  async function send(contact: Contact): Promise<string | null> {
    if (!project?.amMember && !isPrivate) {
      return event
        ? 'Only event members can perform this action'
        : 'Only project members can perform this action';
    }
    try {
      if (isPrivate) await sendEventInvite(contact.agent, detail.id);
      else await shareSpace(contact, event ? 'event' : 'project', title, url);
      return null;
    } catch (e) {
      return explain(e);
    }
  }

  function createPost(): void {
    const prefill = encodeURIComponent(sharePrefill(title, url));
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- the path is resolved here
    void goto(`${resolve('/create/post')}?prefill=${prefill}`);
  }
</script>

{#if shown}
  <ShareUserMenu
    menuTitle={isPrivate ? 'Invite to event' : event ? 'Share event' : 'Share project'}
    submitLabel={isPrivate ? 'Invite' : 'Send'}
    recipientHint={isPrivate
      ? "It's a private event, so they get an invite to accept under Invites."
      : 'They get it as a message and a notification.'}
    copyLinkUrl={url}
    shareTitle={title}
    searchContacts={search}
    submitShare={send}
    createPost={isPrivate ? null : createPost}
  />
{/if}
