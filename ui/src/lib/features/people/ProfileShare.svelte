<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import ShareUserMenu from '#lib/components/overlay/ShareUserMenu.svelte';
  import type { Contact } from '#lib/domain/messages.ts';
  import { explain } from '#lib/services/errors.ts';
  import { contacts } from '#lib/services/messages.ts';
  import { myAgent } from '#lib/services/people.ts';
  import { shareProfile } from '#lib/services/shares.ts';

  import { resolve } from '$app/paths';

  /**
   * Detail Pages Cleanup §5, "Share profile": a link instead of a key (it carries the key, so
   * nobody has to read or type it). Copy it, share it to another app, or send it in a chat.
   */
  let { agent, name }: { agent: string; name: string } = $props();

  const url = $derived(`${location.origin}${resolve('/profile/[agent]', { agent })}`);
  let people: Promise<Contact[]> | null = null;

  async function search(query: string): Promise<Contact[]> {
    people ??= Promise.all([contacts(), myAgent()]).then(([all, me]) =>
      all.filter((c) => c.agent !== me && c.agent !== agent),
    );
    const needle = query.trim().toLowerCase();
    return (await people).filter((c) => c.name.toLowerCase().includes(needle)).slice(0, 8);
  }

  async function send(contact: Contact): Promise<string | null> {
    try {
      await shareProfile(contact, name, url);
      return null;
    } catch (e) {
      return explain(e);
    }
  }
</script>

<ShareUserMenu
  menuTitle={`Share ${name}'s profile`}
  submitLabel={t('Send')}
  recipientHint={t('They get the link as a message.')}
  copyLinkUrl={url}
  shareTitle={name}
  searchContacts={search}
  submitShare={send}
/>
