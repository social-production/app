<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import ShareSheet from '#lib/components/overlay/ShareSheet.svelte';
  import type { Contact } from '#lib/domain/messages.ts';
  import { explain } from '#lib/services/errors.ts';
  import { searchShareContacts } from '#lib/services/shareContacts.ts';
  import { shareTo } from '#lib/services/shares.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  import { shareHost } from './shareHost.svelte.ts';

  /**
   * The share sheet cards and comments open (Owner Notes Mockups §6). Anyone signed in sends
   * it to people as a message, members or not (owner, 2026-11-24); signed out, the link only.
   */
  const subject = $derived(shareHost.subject);
  const signedIn = $derived(connection.current.status === 'connected');
  const url = $derived(subject ? new URL(subject.path, location.origin).href : '');
  const KIND_LABELS = {
    project: 'Project',
    event: 'Event',
    'help request': 'Request',
    thread: 'Thread',
    post: 'Post',
    comment: 'Comment',
    profile: 'Profile',
  } as const;
  let faces = $state<Contact[]>([]);

  $effect(() => {
    if (!subject || !signedIn) return;
    void searchShareContacts('').then(
      (found) => (faces = found),
      () => (faces = []),
    );
  });

  async function send(contact: Contact): Promise<string | null> {
    if (!subject) return null;
    try {
      await shareTo(contact, subject.kind, subject.title, url);
      return null;
    } catch (e) {
      return explain(e);
    }
  }

  function quote(): void {
    if (!subject) return;
    const link = encodeURIComponent(url);
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- the path is resolved here
    void goto(`${resolve('/create/post')}?quote=${link}`);
  }
</script>

<ShareSheet
  open={!!subject}
  title={t('Share')}
  kindLabel={subject ? KIND_LABELS[subject.kind] : ''}
  subject={subject?.title ?? ''}
  {url}
  contacts={signedIn ? faces : []}
  searchContacts={(q) => (signedIn ? searchShareContacts(q) : Promise.resolve([]))}
  submitShare={send}
  recipientHint={signedIn
    ? 'They get it as a message and a notification.'
    : 'Sign in to send it to people here.'}
  onQuote={signedIn && subject?.kind !== 'comment' ? quote : null}
  onClose={() => (shareHost.subject = null)}
/>
