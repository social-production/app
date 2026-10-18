<script lang="ts">
  import type { ResolvedPathname } from '$app/types';
  import type { Snippet } from 'svelte';

  import ScopeGlyph from '#lib/components/icons/ScopeGlyph.svelte';
  import type { SurfaceIconId } from '#lib/components/icons/surface.ts';

  interface Props {
    href: ResolvedPathname;
    icon: SurfaceIconId;
    label: string;
    active?: boolean;
    imageUrl?: string | null;
    trailing?: Snippet;
    onclick?: (() => void) | undefined;
  }

  let { href, icon, label, active = false, imageUrl = null, trailing, onclick }: Props = $props();
</script>

<!-- eslint-disable svelte/no-navigation-without-resolve -- hrefs arrive resolved (ResolvedPathname) -->
<a class="rail-link-row" class:active {href} {onclick}>
  <ScopeGlyph {icon} {imageUrl} />
  <span class="label">{label}</span>
  <span class="trailing">{@render trailing?.()}</span>
</a>

<!-- eslint-enable svelte/no-navigation-without-resolve -->

<style>
  .rail-link-row {
    display: flex;
    gap: 8px;
    align-items: center;
    width: 100%;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    color: var(--text-main);
    font-size: 13px;
    font-weight: 600;
    text-decoration: none;
    background: transparent;
    transition:
      background-color 0.16s ease,
      color 0.16s ease;
  }

  .rail-link-row:hover {
    background: var(--panel-strong);
  }

  .rail-link-row.active {
    background: var(--brand-soft);
    color: var(--brand-strong);
  }

  .label {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .trailing {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .trailing:empty {
    display: none;
  }
</style>
