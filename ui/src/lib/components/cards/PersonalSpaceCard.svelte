<script lang="ts">
  import type { ProjectCard, Stance } from '#lib/domain/projects.ts';

  import PersonalActivityCard from './PersonalActivityCard.svelte';
  import PersonalHelpRequestCard from './PersonalHelpRequestCard.svelte';

  /** web's PersonalFeedCard: the personal-feed card for a space's mode. */
  interface Props {
    card: ProjectCard;
    onSignal: (next: Stance | null) => Promise<void>;
  }

  let { card, onSignal }: Props = $props();
  const authorName = $derived(card.creatorName || 'Unnamed');
</script>

{#if card.mode === 'help-request'}
  <PersonalHelpRequestCard {card} {authorName} {onSignal} />
{:else}
  <PersonalActivityCard {card} {authorName} {onSignal} />
{/if}
