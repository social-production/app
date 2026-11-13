<script lang="ts">
  import type { ProjectCard as Card, Stance } from '#lib/domain/projects.ts';

  import EventCard from './EventCard.svelte';
  import HelpRequestCard from './HelpRequestCard.svelte';
  import ProjectCard from './ProjectCard.svelte';
  import ThreadCard from './ThreadCard.svelte';

  /** web's PublicFeedCard: the card for a space's mode (project, event, help request, thread). */
  interface Props {
    card: Card;
    onSignal: (next: Stance | null) => Promise<void>;
  }

  let { card, onSignal }: Props = $props();
</script>

{#if card.mode === 'help-request'}
  <HelpRequestCard {card} {onSignal} />
{:else if card.mode === 'thread'}
  <ThreadCard {card} {onSignal} />
{:else if card.mode === 'event'}
  <EventCard {card} {onSignal} />
{:else}
  <ProjectCard {card} {onSignal} />
{/if}
