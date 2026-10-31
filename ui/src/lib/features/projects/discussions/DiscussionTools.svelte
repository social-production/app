<script lang="ts">
  import IconMenuButton from '#lib/components/IconMenuButton.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import ShareUserMenu from '#lib/components/overlay/ShareUserMenu.svelte';
  import type { CommentSort } from '#lib/domain/discussions.ts';
  import type { Contact } from '#lib/domain/messages.ts';
  import { shareDiscussion } from '#lib/services/discussions.ts';
  import { explain } from '#lib/services/errors.ts';
  import { contacts } from '#lib/services/messages.ts';
  import { myAgent } from '#lib/services/people.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  /** The detail footer's tools on web's thread and post pages: Share (signed in) and Sort. */
  interface Props {
    kind: 'thread' | 'post';
    title: string;
    sortMode: CommentSort;
  }

  let { kind, title, sortMode = $bindable() }: Props = $props();

  const sortOptions: { value: CommentSort; label: string }[] = [
    { value: 'oldest', label: 'Oldest first' },
    { value: 'newest', label: 'Newest first' },
    { value: 'top', label: 'Top voted' },
  ];
  const signedIn = $derived(connection.current.status === 'connected');
  const url = $derived(`${location.origin}${location.pathname}`);
  let people: Promise<Contact[]> | null = null;

  async function search(query: string): Promise<Contact[]> {
    people ??= Promise.all([contacts(), myAgent()]).then(([all, me]) =>
      all.filter((c) => c.agent !== me),
    );
    const needle = query.trim().toLowerCase();
    return (await people).filter((c) => c.name.toLowerCase().includes(needle)).slice(0, 8);
  }

  async function send(contact: Contact): Promise<string | null> {
    try {
      await shareDiscussion(contact, kind, title, url);
      return null;
    } catch (e) {
      return explain(e);
    }
  }
</script>

<div class="detail-tools">
  {#if signedIn}
    <span class="detail-tool">
      <ShareUserMenu
        copyLinkUrl={url}
        menuTitle={kind === 'thread' ? 'Share thread' : 'Share post'}
        compact
        shareTitle={title}
        searchContacts={search}
        submitShare={send}
      />
    </span>
  {/if}
  <span class="detail-tool">
    <IconMenuButton
      value={sortMode}
      ariaLabel="Sort comments"
      defaultValue="oldest"
      options={sortOptions}
      onChange={(v) => (sortMode = v)}
    >
      <ToolbarIcon name="sort" />
    </IconMenuButton>
  </span>
</div>
