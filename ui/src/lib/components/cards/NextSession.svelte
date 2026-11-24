<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { nextGoing, nextMore, nextWhen, type NextActivity } from '#lib/domain/nextSession.ts';

  /**
   * A card's next planned activity (owner, 2026-11-24, from the Meetup mockup's card): "Next
   * Thu 6:00 pm · 2 of 4 going", on a soft row that opens the activity on its page.
   */
  import '#lib/components/styles/ux/nextsession.css';

  let { next, href }: { next: NextActivity; href: string } = $props();

  const going = $derived(nextGoing(next));
  const more = $derived(nextMore(next));
</script>

<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- the card's resolved page -->
<a class="next-session" href={`${href}?activity=${encodeURIComponent(next.id)}`}>
  <svg aria-hidden="true" viewBox="0 0 24 24"
    ><rect x="4" y="5" width="16" height="15" rx="2" /><path d="M4 9h16M8 3v4M16 3v4" /></svg
  >
  <span class="next-session-text">
    {t('Next')} <b>{nextWhen(next)}</b>
    {#if going}<span class="next-session-going">· {going}</span>{/if}
  </span>
  {#if more}<span class="next-session-more">{more}</span>{/if}
</a>
