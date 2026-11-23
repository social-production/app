<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import ShareUserMenu from '#lib/components/overlay/ShareUserMenu.svelte';
  import type { Contact } from '#lib/domain/messages.ts';
  import { searchShareContacts } from '#lib/services/shareContacts.ts';
  import { explain } from '#lib/services/errors.ts';
  import { shareProfile } from '#lib/services/shares.ts';

  import { resolve } from '$app/paths';

  /**
   * Detail Pages Cleanup §5, "Share profile": a link instead of a key (it carries the key, so
   * nobody has to read or type it). Copy it, share it to another app, or send it in a chat.
   */
  let {
    agent,
    name,
    compact = false,
  }: { agent: string; name: string; compact?: boolean } = $props();

  const url = $derived(`${location.origin}${resolve('/profile/[agent]', { agent })}`);

  const search = (query: string): Promise<Contact[]> => searchShareContacts(query, [agent]);

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
  menuTitle={t("Share {name}'s profile", { name })}
  submitLabel={t('Send')}
  recipientHint={t('They get the link as a message.')}
  copyLinkUrl={url}
  shareTitle={name}
  searchContacts={search}
  submitShare={send}
  {compact}
/>
