<script lang="ts">
  import type { ProjectCard as Card, Stance } from '#lib/domain/projects.ts';

  import EventCard from './EventCard.svelte';
  import HelpRequestCard from './HelpRequestCard.svelte';
  import ProjectCard from './ProjectCard.svelte';

  /** web's PublicFeedCard: the card for a space's mode (project, event, help request). */
  interface Props {
    card: Card;
    onSignal: (next: Stance | null) => Promise<void>;
  }

  let { card, onSignal }: Props = $props();
  const authorName = $derived(card.creatorName || 'Unnamed');
</script>

{#if card.mode === 'help-request'}
  <HelpRequestCard {card} {authorName} {onSignal} />
{:else if card.mode === 'event'}
  <EventCard {card} {authorName} {onSignal} />
{:else}
  <ProjectCard {card} {authorName} {onSignal} />
{/if}
