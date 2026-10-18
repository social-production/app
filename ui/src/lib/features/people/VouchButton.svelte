<script lang="ts">
  import type { Person } from '#lib/domain/people.ts';

  /** web's "Vouch" action button (profile action row). */
  interface Props {
    person: Person;
    viewer: Person | undefined;
    busy: boolean;
    onVouch: () => void;
  }

  let { person, viewer, busy, onVouch }: Props = $props();

  const vouched = $derived(!!viewer && person.standing.vouchers.includes(viewer.agent));
  const canVouch = $derived(
    viewer?.standing.level === 'Full' && person.standing.level === 'New' && !vouched,
  );
  const title = $derived(
    vouched
      ? 'You vouched for them'
      : person.standing.level !== 'New'
        ? 'Already a full member'
        : canVouch
          ? 'Vouch'
          : 'Vouching opens once your own votes count fully.',
  );
</script>

<button
  aria-label={vouched
    ? `You vouched for ${person.profile?.name ?? 'them'}`
    : `Vouch for ${person.profile?.name ?? 'them'}`}
  aria-pressed={vouched}
  class="action vouch"
  class:on={vouched}
  disabled={busy || !canVouch}
  {title}
  type="button"
  onclick={onVouch}
>
  <svg aria-hidden="true" viewBox="0 0 24 24" width="18" height="18"
    ><path
      d="M12 3l7 3v5c0 4.5-3 8.3-7 10-4-1.7-7-5.5-7-10V6l7-3Z"
      fill="none"
      stroke="currentColor"
      stroke-width="1.8"
      stroke-linejoin="round"
    /><path
      d="M8.5 12l2.5 2.5 4.5-5"
      fill="none"
      stroke="currentColor"
      stroke-width="1.8"
      stroke-linecap="round"
      stroke-linejoin="round"
    /></svg
  >
  <span class="action-text">{vouched ? 'Vouched' : 'Vouch'}</span>
</button>
