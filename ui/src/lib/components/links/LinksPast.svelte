<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import type { LinkRequest, ProjectLink } from '#lib/domain/links.ts';
  import { pastRows } from '#lib/domain/linksPast.ts';

  import LinkCard from './LinkCard.svelte';
  import LinkVoteCard from './LinkVoteCard.svelte';

  import '#lib/components/styles/links.css';

  /**
   * web's Past links and Past link votes as one fold (Polish Pass mockups §6): each ended link
   * says why it ended, each failed vote who said no, without the percentages; tapping one
   * shows the link or how the vote went.
   */
  interface Props {
    links: ProjectLink[];
    votes: LinkRequest[];
    names: Record<string, string>;
  }

  let { links, votes, names }: Props = $props();

  let open = $state(false);
  let expanded = $state<Record<string, boolean>>({});
  const rows = $derived(pastRows(links, votes));

  const toggle = (id: string) => (expanded = { ...expanded, [id]: !expanded[id] });
</script>

{#if rows.length > 0}
  <section class="vote-archive">
    <button
      class="links-disclose"
      type="button"
      aria-expanded={open}
      onclick={() => (open = !open)}
    >
      <span>{t('Past links')} · {rows.length}</span>
      <svg aria-hidden="true" viewBox="0 0 24 24"><path d="m7 10 5 5 5-5" /></svg>
    </button>
    {#if open}
      <div class="request-list">
        {#each rows as row (row.key)}
          <LinkCard
            title={row.title}
            metaLine={row.meta}
            outcomeLabel={row.tag}
            outcomeTone="blocked"
            icon="link"
            projectId={row.targetId}
            expanded={!!expanded[row.key]}
            domId={`link-${row.key}`}
            onToggle={() => toggle(row.key)}
          >
            {#if row.request}
              <LinkVoteCard request={row.request} proposer={names[row.request.proposedBy] ?? '…'} />
            {:else if row.link}
              <p class="detail-description">
                {row.link.create.target.description || row.link.create.summary}
              </p>
            {/if}
          </LinkCard>
        {/each}
      </div>
    {/if}
  </section>
{/if}
