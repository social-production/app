<script lang="ts">
  import ContentMetaRow from '#lib/components/cards/ContentMetaRow.svelte';
  import RoundPlusButton from '#lib/components/RoundPlusButton.svelte';
  import type { ProjectUpdate } from '#lib/domain/governance.ts';

  import { resolve } from '$app/paths';

  import '#lib/components/styles/governance.css';

  /** web's ProjectUpdatesSection with DetailUpdateCard: passed updates, newest first. */
  interface Props {
    updates: ProjectUpdate[];
    names: Record<string, string>;
    canPropose: boolean;
    onAdd: () => void;
  }

  let { updates, names, canPropose, onAdd }: Props = $props();
</script>

<section class="updates-shell overview-updates" id="updates">
  <div class="updates-title-row">
    <h2>Updates</h2>
    {#if canPropose}<RoundPlusButton
        label="Add update"
        ariaLabel="Add update"
        onclick={onAdd}
      />{/if}
  </div>
  <div class:scrollable={updates.length > 4} class="stack updates-list">
    {#each updates as update (update.proposal)}
      <article id={`update-${update.proposal}`} class="update-card">
        <p class="update-body">{update.body}</p>
        <div class="update-meta">
          <ContentMetaRow
            authorName={names[update.author] ?? '…'}
            authorHref={resolve('/profile/[agent]', { agent: update.author })}
            createdAt={update.at}
          />
        </div>
      </article>
    {:else}
      <div class="empty-card"><p>No updates yet.</p></div>
    {/each}
  </div>
</section>
