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
  <span class="rail-glyph" data-icon={icon}><ScopeGlyph {icon} {imageUrl} /></span>
  <span class="label">{label}</span>
  <span class="trailing">{@render trailing?.()}</span>
</a>

<!-- eslint-enable svelte/no-navigation-without-resolve -->

<style>
  /* A soft square glyph (Plan, Phase and Feedback Screens, 5): # for a channel, people for
     a community; the open row is tinted. */
  .rail-glyph {
    display: grid;
    flex: none;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: 10px;
    background: var(--panel-hover);
    color: var(--text-soft);
  }

  .rail-glyph[data-icon='channel'] {
    background: color-mix(in srgb, #3f70ab 16%, transparent);
    color: color-mix(in srgb, #3f70ab 70%, var(--text-main));
  }

  .rail-glyph[data-icon='community'] {
    background: var(--brand-soft);
    color: var(--brand-strong);
  }

  .rail-link-row {
    display: flex;
    gap: 11px;
    align-items: center;
    width: 100%;
    padding: 6px 8px;
    border-radius: 14px;
    color: var(--text-main);
    font-size: 14.5px;
    font-weight: 600;
    text-decoration: none;
    background: transparent;
    transition:
      background-color 0.16s ease,
      color 0.16s ease;
  }

  .rail-link-row:hover {
    background: var(--panel-hover);
  }

  .rail-link-row.active {
    background: var(--brand-soft);
    color: var(--brand-strong);
  }

  .label {
    flex: 1;
    min-width: 0;
    overflow-wrap: anywhere;
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
