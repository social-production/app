<script lang="ts">
  import { statusLabel, type LinkRequest, type ProjectLink } from '#lib/domain/links.ts';

  import LinkCard from './LinkCard.svelte';
  import LinkVoteCard from './LinkVoteCard.svelte';

  import '#lib/components/styles/links.css';

  /**
   * web's Past links and Past link votes as one fold (Polish Pass mockups, 2026-11-16): each
   * ended link and each decided vote is a row with a tag; tapping one shows how it was decided.
   */
  interface Props {
    links: ProjectLink[];
    votes: LinkRequest[];
    names: Record<string, string>;
  }

  let { links, votes, names }: Props = $props();

  let open = $state(false);
  let expanded = $state<Record<string, boolean>>({});
  const count = $derived(links.length + votes.length);

  function tone(r: LinkRequest): 'approved' | 'blocked' | 'pending' {
    if (r.status === 'Approved') return 'approved';
    return r.status === 'Rejected' ? 'blocked' : 'pending';
  }

  const toggle = (id: string) => (expanded = { ...expanded, [id]: !expanded[id] });
</script>

{#if count > 0}
  <section class="vote-archive">
    <button
      class="links-disclose"
      type="button"
      aria-expanded={open}
      onclick={() => (open = !open)}
    >
      <span>Past links · {count}</span>
      <svg aria-hidden="true" viewBox="0 0 24 24"><path d="m7 10 5 5 5-5" /></svg>
    </button>
    {#if open}
      <div class="request-list">
        {#each links as link (link.id)}
          <LinkCard
            title={link.create.target.title}
            metaLine="Link ended"
            outcomeLabel="Ended"
            outcomeTone="blocked"
            projectId={link.create.target.id}
            expanded={!!expanded[link.id]}
            domId={`link-${link.id}`}
            onToggle={() => toggle(link.id)}
          >
            <p class="detail-description">
              {link.create.target.description || link.create.summary}
            </p>
          </LinkCard>
        {/each}
        {#each votes as request (request.id)}
          <LinkCard
            title={request.target.title}
            metaLine={`${request.sever ? 'Vote to end the link' : 'Link vote'} · ${names[request.proposedBy] ?? '…'}`}
            outcomeLabel={statusLabel(request)}
            outcomeTone={tone(request)}
            projectId={request.target.id}
            expanded={!!expanded[request.id]}
            domId={`link-request-${request.id}`}
            onToggle={() => toggle(request.id)}
          >
            <LinkVoteCard {request} proposer={names[request.proposedBy] ?? '…'} />
          </LinkCard>
        {/each}
      </div>
    {/if}
  </section>
{/if}
