<script lang="ts">
  import type { FeedContext } from '#lib/domain/homeFeed.ts';
  import type { ProjectCard as Card, Stance } from '#lib/domain/projects.ts';

  import EventCard from './EventCard.svelte';
  import HelpRequestCard from './HelpRequestCard.svelte';
  import ProjectCard from './ProjectCard.svelte';
  import ThreadCard from './ThreadCard.svelte';

  /** web's PublicFeedCard: the card for a space's mode (project, event, help request, thread). */
  interface Props {
    card: Card;
    onSignal: (next: Stance | null) => Promise<void>;
    /** Home's "Maya Kim proposed this" line (ADR 0107). */
    context?: FeedContext | null;
  }

  let { card, onSignal, context = null }: Props = $props();
</script>

{#if card.mode === 'help-request'}
  <HelpRequestCard {card} {onSignal} {context} />
{:else if card.mode === 'thread'}
  <ThreadCard {card} {onSignal} {context} />
{:else if card.mode === 'event'}
  <EventCard {card} {onSignal} {context} />
{:else}
  <ProjectCard {card} {onSignal} {context} />
{/if}
