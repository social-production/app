<script lang="ts">
  import type { Snippet } from 'svelte';

  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import { placeOnMap } from '#lib/components/map/placeOnMap.ts';
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
    /** web's historyMode: the past row's badge and rating line; the sheet gets `children`. */
    history?: { badge: string; tone: 'open' | 'need' | 'locked'; rating: string } | null;
    /** web's `agendaChip` (a service's Open / Taken / Booked…), in place of the fill chip. */
    chip?: { label: string; tone: 'locked' | 'need' | 'full' | 'open' } | null;
    /** web's `onSelect`: tapping the row calls this instead of opening the card. */
    onSelect?: (() => void) | null;
    children?: Snippet;
  }

  let p: Props = $props();

  // A highlighted activity (web's ?activity= link) opens once.
  // svelte-ignore state_referenced_locally
  let open = $state(p.highlighted);
  let copied = $state(false);

  const a = $derived(p.activity);
  const chip = $derived(p.chip ?? fillChip(a));
  const place = $derived(placeLabel(a));
  // The row's faces (ADR 0102): the first three people in, and "Join" while a role has room.
  const people = $derived(a.roles.flatMap((r) => r.people));
  const joinable = $derived(
    !p.history &&
      !p.chip &&
      !p.readOnly &&
      a.rolesOpen &&
      a.myRole === null &&
      a.roles.some((r) => r.maximum === null || r.people.length < r.maximum),
  );
  const zone = $derived(zoneOf(new Date(a.startsAt / 1000)));
  const until = $derived(multiDayEnd(a.startsAt, a.endsAt));
  const shortDay = $derived(
    new Date(a.startsAt / 1000).toLocaleDateString(undefined, { day: 'numeric', month: 'short' }),
  );
  const mapHref = $derived(
    `https://www.openstreetmap.org/search?query=${encodeURIComponent(a.place)}`,
  );

  /** web's `showOnMap`: a plain click opens the app's map on the place (ADR 0085). */
  const map = placeOnMap();
  function showOnMap(event: MouseEvent): void {
    if (!map || event.metaKey || event.ctrlKey || event.shiftKey) return;
    event.preventDefault();
    open = false;
    map.show(a.place);
  }

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
    onclick={() => (p.onSelect ? p.onSelect() : (open = true))}
  >
    <span class="agenda-time">
      {#if p.history}<span>{shortDay}</span>{/if}
      <span>{clockTime(a.startsAt)}</span>
      {#if !p.history}
        <span class="agenda-end">{clockTime(a.endsAt)}{zone ? ` ${zone}` : ''}</span>
      {/if}
    </span>
    <span class="agenda-main">
      <strong>{a.title}</strong>
      <span
        >{#if until}Until {until} ·{/if}
        {place || 'Place to be confirmed'}{#if p.history}
          · {p.history.rating}{/if}</span
      >
    </span>
    <span class="agenda-foot">
      {#if !p.history && people.length > 0}
        <span class="agenda-people" aria-hidden="true">
          {#each people.slice(0, 3) as person (person)}
            <AvatarBadge size="sm" name={p.names[person] ?? '…'} />
          {/each}
        </span>
      {/if}
      {#if p.history}
        <span class={`agenda-chip ${p.history.tone}`}>{p.history.badge}</span>
      {:else}
        <span class={`agenda-chip ${chip.tone}`}>{chip.label}</span>
      {/if}
      {#if joinable}<span class="agenda-join">Join</span>{/if}
    </span>
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
      <p class="sheet-when sheet-line">
        <ToolbarIcon name="clock" /><span>{dateLine(a.startsAt, a.endsAt)}</span>
      </p>
      {#if place}
        {#if !a.online && a.place}
          <!-- eslint-disable svelte/no-navigation-without-resolve -- an outside map -->
          <a
            class="sheet-place sheet-line map-link"
            href={mapHref}
            title="Show on map"
            target="_blank"
            rel="noopener noreferrer"
            onclick={showOnMap}><ToolbarIcon name="map-pin" /><span>{place}</span></a
          >
          <!-- eslint-enable svelte/no-navigation-without-resolve -->
        {:else}
          <p class="sheet-place sheet-line">
            <ToolbarIcon name={a.online ? 'globe' : 'map-pin'} /><span>{place}</span>
          </p>
        {/if}
      {/if}
      {#if a.online && a.meetingUrl}
        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- an outside meeting -->
        <a class="meeting-link" href={a.meetingUrl} target="_blank" rel="noopener noreferrer"
          >Join meeting</a
        >
      {/if}
      {#if p.planStage}
        <p class="activity-stage sheet-line">
          <ToolbarIcon name="layers" /><span>{p.planStage}</span>
        </p>
      {/if}
      {#if a.note.trim()}
        <section class="sheet-section">
          <h3>Description</h3>
          <p class="activity-note">{a.note.trim()}</p>
        </section>
      {/if}
      {#if a.roles.length > 0}
        <ActivityRoles
          past={!!p.history}
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
      {@render p.children?.()}
      <div class="sheet-actions">
        <button
          class="sheet-pill share-activity"
          type="button"
          title="Copy a link to this activity"
          onclick={share}
        >
          <ToolbarIcon name="link" />{copied ? 'Link copied' : 'Share activity'}
        </button>
        {#if !a.online && a.place}
          <!-- eslint-disable svelte/no-navigation-without-resolve -- an outside map -->
          <a
            class="sheet-pill"
            href={mapHref}
            target="_blank"
            rel="noopener noreferrer"
            onclick={showOnMap}><ToolbarIcon name="map-pin" />Show on map</a
          >
          <!-- eslint-enable svelte/no-navigation-without-resolve -->
        {/if}
      </div>
    </div>
  </OverlaySheet>
</div>
