import { scopeImageUrl } from '#lib/services/scopeImages.ts';
import { findScope } from '#lib/services/scopes.ts';

/**
 * Pictures of the channels and communities tags name (ADR 0109), looked up once per slug the
 * first time a tag shows and kept for the session. The shell puts `of` in context.
 */
export class ScopePictures {
  private urls = $state<Record<string, string | null>>({});
  // Not reactive on purpose: it only stops asking twice.
  // eslint-disable-next-line svelte/prefer-svelte-reactivity
  private asked = new Set<string>();

  of = (slug: string): string | null => {
    if (!this.asked.has(slug)) {
      this.asked.add(slug);
      void this.load(slug);
    }
    return this.urls[slug] ?? null;
  };

  private async load(slug: string): Promise<void> {
    const scope = await findScope(slug).catch(() => null);
    if (!scope?.picture) return;
    this.urls[slug] = await scopeImageUrl(scope.picture);
  }
}
