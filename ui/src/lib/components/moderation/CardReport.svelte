<script lang="ts">
  import { cardModeration, itemLabelOf } from '#lib/domain/moderation.ts';
  import type { ProjectCard } from '#lib/domain/projects.ts';

  import ReportControl from './ReportControl.svelte';

  /** web's ReportControl on a feed card: the index's state, the card's author and link. */
  let { card, href = null }: { card: ProjectCard; href?: string | null } = $props();

  const copyUrl = $derived(
    href && typeof window !== 'undefined' ? `${location.origin}${href}` : '',
  );
</script>

<ReportControl
  id={card.id}
  itemLabel={itemLabelOf(card.mode)}
  known={cardModeration(card.moderation)}
  owner={card.creator}
  {copyUrl}
/>
