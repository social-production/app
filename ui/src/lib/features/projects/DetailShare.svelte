<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import ShareUserMenu from '#lib/components/overlay/ShareUserMenu.svelte';
  import type { Contact } from '#lib/domain/messages.ts';
  import { isEvent } from '#lib/domain/projects.ts';
  import { searchShareContacts } from '#lib/services/shareContacts.ts';
  import { explain } from '#lib/services/errors.ts';
  import { sendEventInvite } from '#lib/services/events.ts';
  import { quotePrefill, shareSpace } from '#lib/services/shares.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  import type { DetailState } from './detailState.svelte.ts';

  /**
   * web's "Share +" on ProjectOverviewHeader / EventOverviewHeader. Detail Pages Cleanup §4:
   * Share is always on the dock; anyone signed in may send it (owner, 2026-11-24). Private
   * events: organizers only, as "Invite to event" (an invite to accept, no Create post).
   */
  let { detail, label = 'Share' }: { detail: DetailState; label?: string } = $props();

  const project = $derived(detail.project);
  const event = $derived(isEvent(project?.charter?.mode));
  const isPrivate = $derived(event && project?.audience !== 'public');
  const signedIn = $derived(connection.current.status === 'connected');
  const shown = $derived(signedIn && !!project && (!isPrivate || !!project.amOrganizer));
  const url = $derived(`${location.origin}${location.pathname}`);
  const title = $derived(project?.charter?.title ?? '');

  const search = (query: string): Promise<Contact[]> => searchShareContacts(query);

  async function send(contact: Contact): Promise<string | null> {
    try {
      if (isPrivate) await sendEventInvite(contact.agent, detail.id);
      else await shareSpace(contact, event ? 'event' : 'project', title, url);
      return null;
    } catch (e) {
      return explain(e);
    }
  }

  function createPost(): void {
    const prefill = encodeURIComponent(quotePrefill(url));
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- the path is resolved here
    void goto(`${resolve('/create/post')}?prefill=${prefill}`);
  }
</script>

{#if shown}
  <ShareUserMenu
    menuTitle={isPrivate ? t('Invite to event') : event ? t('Share event') : t('Share project')}
    submitLabel={isPrivate ? t('Invite') : t('Send')}
    recipientHint={isPrivate
      ? t("It's a private event, so they get an invite to accept under Invites.")
      : t('They get it as a message and a notification.')}
    buttonLabel={label}
    copyLinkUrl={url}
    shareTitle={title}
    kindLabel={event ? 'Event' : 'Project'}
    searchContacts={search}
    submitShare={send}
    createPost={isPrivate ? null : createPost}
  />
{/if}
