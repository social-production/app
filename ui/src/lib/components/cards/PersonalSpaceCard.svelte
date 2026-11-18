<script lang="ts">
  import type { FeedContext } from '#lib/domain/homeFeed.ts';
  import type { ProjectCard, Stance } from '#lib/domain/projects.ts';

  import PersonalPostCard from './PersonalPostCard.svelte';
  import SpaceCard from './SpaceCard.svelte';

  /**
   * A personal-feed item (Home's people you follow, a profile): posts keep their author row;
   * everything else is the Global row (Profiles and Comments mockups §1), with Home's
   * "Maya Kim proposed this" line when `context` is given.
   */
  interface Props {
    card: ProjectCard;
    onSignal: (next: Stance | null) => Promise<void>;
    context?: FeedContext | null;
  }

  let { card, onSignal, context = null }: Props = $props();
</script>

{#if card.mode === 'post'}
  <PersonalPostCard {card} authorName={card.creatorName || 'Unnamed'} {onSignal} />
{:else}
  <SpaceCard {card} {onSignal} {context} />
{/if}
