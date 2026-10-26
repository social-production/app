<script lang="ts">
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { fillChip, placeLabel, signupTone, type Activity } from '#lib/domain/activities.ts';
  import { clockTime, dateLine, multiDayEnd, zoneOf } from '#lib/domain/activityTime.ts';

  import ActivityRoles from './ActivityRoles.svelte';

  import '#lib/components/styles/activities.css';

  /**
   * web's CollapsibleActivityCard in agenda mode: a row (time | title + place | fill chip)
   * that opens the activity's sheet (place, when, plan stage, description, roles, share).
   */
  interface Props {
    activity: Activity;
    names: Record<string, string>;
    me: string;
    /** The linked plan stage's label ("Production: Build frames"), if any. */
    planStage: string | null;
    highlighted: boolean;
    readOnly: boolean;
    canSuggest: boolean;
    onCommit: (role: number | null) => void;
    onSuggest: (role: number, agent: string) => void;
    onDecline: (suggestion: string) => void;
    searchPeople: (q: string) => Promise<{ agent: string; name: string }[]>;
  }

  let p: Props = $props();

  // A highlighted activity (web's ?activity= link) opens once.
  // svelte-ignore state_referenced_locally
  let open = $state(p.highlighted);
  let copied = $state(false);

  const a = $derived(p.activity);
  const chip = $derived(fillChip(a));
  const place = $derived(placeLabel(a));
  const zone = $derived(zoneOf(new Date(a.startsAt / 1000)));
  const until = $derived(multiDayEnd(a.startsAt, a.endsAt));
  const mapHref = $derived(
    `https://www.openstreetmap.org/search?query=${encodeURIComponent(a.place)}`,
  );

  /** web's shareLink: the system share sheet, else the link copied. */
  async function share(): Promise<void> {
    const url = `${location.origin}${location.pathname}?activity=${a.id}`;
    try {
      if (navigator.share) await navigator.share({ url, title: a.title });
      else {
        await navigator.clipboard.writeText(url);
        copied = true;
      }
    } catch {
      // Dismissed or blocked: nothing to do.
    }
  }
</script>

<div
  class={`activity-card-shell agenda signup-${signupTone(a)}`}
  class:expanded={open}
  class:highlighted={p.highlighted}
>
  <button
    id={`activity-${a.id}`}
    class="agenda-row"
    type="button"
    title={a.title}
    onclick={() => (open = true)}
  >
    <span class="agenda-time">
      <span>{clockTime(a.startsAt)}</span>
      <span class="agenda-end">{clockTime(a.endsAt)}{zone ? ` ${zone}` : ''}</span>
    </span>
    <span class="agenda-main">
      <strong>{a.title}</strong>
      <span
        >{#if until}Until {until} ·{/if}
        {place || 'Place to be confirmed'}</span
      >
    </span>
    <span class={`agenda-chip ${chip.tone}`}>{chip.label}</span>
  </button>

  <OverlaySheet
    activity
    elevated
    rootClass="activity-ui"
    {open}
    title={a.title}
    onClose={() => (open = false)}
  >
    <div class="activity-body">
      {#if place}
        {#if !a.online && a.place}
          <!-- eslint-disable svelte/no-navigation-without-resolve -- an outside map -->
          <a
            class="sheet-place map-link"
            href={mapHref}
            title="Show on map"
            target="_blank"
            rel="noopener noreferrer">{place}</a
          >
          <!-- eslint-enable svelte/no-navigation-without-resolve -->
        {:else}
          <p class="sheet-place">{place}</p>
        {/if}
      {/if}
      {#if a.online && a.meetingUrl}
        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- an outside meeting -->
        <a class="meeting-link" href={a.meetingUrl} target="_blank" rel="noopener noreferrer"
          >Join meeting</a
        >
      {/if}
      <p class="sheet-when">{dateLine(a.startsAt, a.endsAt)}</p>
      {#if p.planStage}
        <p class="activity-stage">{p.planStage}</p>
      {/if}
      {#if a.note.trim()}
        <section class="sheet-section">
          <h3>Description</h3>
          <p class="activity-note">{a.note.trim()}</p>
        </section>
      {/if}
      {#if a.roles.length > 0}
        <ActivityRoles
          activity={a}
          names={p.names}
          me={p.me}
          readOnly={p.readOnly}
          canSuggest={p.canSuggest}
          onCommit={p.onCommit}
          onSuggest={p.onSuggest}
          onDecline={p.onDecline}
          searchPeople={p.searchPeople}
        />
      {/if}
      <button
        class="text-button share-activity"
        type="button"
        title="Copy a link to this activity"
        onclick={share}
      >
        {copied ? 'Link copied' : 'Share activity'}
      </button>
    </div>
  </OverlaySheet>
</div>
