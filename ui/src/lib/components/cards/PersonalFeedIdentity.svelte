<script lang="ts">
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';

  import type { ResolvedPathname } from '$app/types';

  import '#lib/components/styles/cards.css';

  /** web's PersonalFeedIdentity: the author's avatar and name heading a personal-feed card. */
  interface Props {
    name: string;
    href: ResolvedPathname;
    action?: string;
  }

  let { name, href, action = '' }: Props = $props();
</script>

<div class="identity">
  <AvatarBadge size="sm" {name} />
  <div class="identity-copy">
    {#if action}
      <p class="action-note">{action}</p>
    {/if}
    <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href arrives resolved (ResolvedPathname) -->
    <a class="header-name" {href} title={name}>{name}</a>
  </div>
</div>
