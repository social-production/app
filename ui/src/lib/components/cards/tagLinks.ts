import type { SurfaceIconId } from '#lib/components/icons/surface.ts';
import type { TagRef } from '#lib/domain/scopes.ts';

import { resolve } from '$app/paths';

/** web's TagList iconFor: free tags (from before channels) have no icon. */
export function tagIcon(tag: TagRef): SurfaceIconId | null {
  if (tag.kind === 'community') return 'community';
  if (tag.kind === 'channel') return tag.slug === 'platform' ? 'platform' : 'channel';
  return null;
}

/** web's TagList hrefFor; a free tag searches for itself. */
export function tagHref(tag: TagRef): string {
  if (tag.kind === 'channel' && tag.slug === 'platform') return resolve('/platform');
  if (tag.kind === 'community') return resolve('/communities/[slug]', { slug: tag.slug });
  if (tag.kind === 'channel') return resolve('/channels/[slug]', { slug: tag.slug });
  return resolve(`/search?q=${encodeURIComponent(tag.tag)}`);
}

/** web's ScopeSheet kindLabel. */
export function tagKindLabel(tag: TagRef): string {
  if (tag.kind === 'channel' && tag.slug === 'platform') return 'Platform';
  if (tag.kind === 'community') return 'Community';
  return tag.kind === 'channel' ? 'Channel' : 'Tag';
}
