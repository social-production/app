<script lang="ts">
  import ActivityCalendar from '#lib/components/activities/ActivityCalendar.svelte';
  import PhaseWorkToolbar from '#lib/components/governance/PhaseWorkToolbar.svelte';
  import RoundPlusButton from '#lib/components/RoundPlusButton.svelte';
  import { firstSlotOn, serviceSlots } from '#lib/domain/availability.ts';
  import type { RequestSettings } from '#lib/domain/projects.ts';
  import {
    countsLine,
    creatorChip,
    openSlots,
    slotActivity,
    slotChip,
  } from '#lib/domain/serviceAgenda.ts';
  import { bookRequest } from '#lib/services/activities.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  import { ActivitiesState } from '../activitiesState.svelte.ts';
  import type { DetailState } from '../detailState.svelte.ts';
  import { AvailabilityState } from './availabilityState.svelte.ts';
  import AvailabilitySheet from './AvailabilitySheet.svelte';
  import RequestComposer from './RequestComposer.svelte';
  import RequestSettingsSheet from './RequestSettingsSheet.svelte';
  import RequestsSheet from './RequestsSheet.svelte';
  import ServiceActivity from './ServiceActivity.svelte';
  import ServiceHistory from './ServiceHistory.svelte';
  import { ServiceRequestsState } from './serviceRequestsState.svelte.ts';

  import '#lib/components/styles/activities.css';
  import '#lib/components/styles/service-requests.css';

  /**
   * A personal service's Activity phase (web's IndividualServicePhaseOne, ADR 0076/0077): the
   * owner note, the calendar of listed slots, Add availability, Requests, Request settings and
   * Request service, the Activity section and Past activity.
   */
  let { detail }: { detail: DetailState } = $props();

  const me = $derived(connection.current.status === 'connected' ? connection.current.agent : '');
  const project = $derived(detail.project);
  const creator = $derived(!!me && project?.creator === me);
  const place = $derived(project?.charter?.location_label ?? '');
  // svelte-ignore state_referenced_locally
  const activities = new ActivitiesState(detail.id, () => me);
  // svelte-ignore state_referenced_locally
  const availability = new AvailabilityState(detail.id);
  // svelte-ignore state_referenced_locally
  const requests = new ServiceRequestsState(
    detail.id,
    () => me,
    () => creator,
    async (r) => {
      if (r.startsAt === null || r.endsAt === null) return;
      await bookRequest(detail.id, { ...r, startsAt: r.startsAt, endsAt: r.endsAt });
      await activities.load();
    },
  );
  const fallback: RequestSettings = { enabled: true, mode: 'both', offSchedule: true };
  const settings = $derived(project?.requests ?? fallback);
  const requestsOff = $derived(!settings.enabled);
  const usesCalendar = $derived(settings.mode !== 'direct');
  const highlighted = $derived(page.url.searchParams.get('request'));
  const canRequest = $derived(settings.enabled && !!me && !creator);
  let previewDay = $state('');
  let notice = $state('');

  const slots = $derived(serviceSlots(availability.rules, activities.activities, me));
  const slotRows = $derived(
    slots.map((s) => slotActivity(s, creator, project?.creator ?? '', place, activities.names)),
  );
  const accepted = $derived(
    requests.shown.filter(
      (r) => r.status === 'accepted' && (r.endsAt === null || r.endsAt > Date.now() * 1000),
    ).length,
  );
  const counts = $derived(
    countsLine({
      usesCalendar,
      requestsOff,
      creator,
      slots,
      accepted,
      live: activities.live.length,
    }),
  );
  const rows = $derived(
    creator ? activities.live : slotRows.filter((a) => a.endsAt > Date.now() * 1000),
  );

  $effect(() => {
    activities.highlighted = page.url.searchParams.get('activity');
    void activities.load();
    void availability.load();
  });
  $effect(() => {
    if (me) void requests.load();
  });
  // A booking the chat accepted but the project never got is made when the creator looks.
  $effect(() => {
    if (creator && requests.loaded && activities.loaded) {
      void requests.heal(activities.activities.flatMap((a) => a.booking?.request ?? []));
    }
  });
  $effect(() => {
    if (highlighted) requests.listOpen = true;
  });

  /** web's `needsSignIn`. */
  function needsSignIn(): boolean {
    if (creator) return false;
    if (requestsOff) {
      notice = 'This service is not taking requests right now.';
      return true;
    }
    if (me) return false;
    void goto(resolve('/onboarding'));
    return true;
  }

  /** web's `openCalendarComposerForActivity`: a bar or row requests its open slot. */
  function requestSlot(id: string): void {
    notice = '';
    if (creator || !usesCalendar || needsSignIn()) return;
    const slot = slots.find((s) => s.id === id);
    if (slot?.viewerBooked) {
      requests.listOpen = true;
    } else if (slot?.held) {
      notice = 'That slot is taken. Pick an open one.';
    } else if (slot) {
      requests.openForSlot(slot);
    }
  }

  /** web's `openDockedServiceRequest`. */
  function openRequest(): void {
    notice = '';
    if (!usesCalendar || !previewDay) {
      requests.openComposer(settings.mode);
      return;
    }
    const slot = firstSlotOn(slots, previewDay);
    if (slot) requests.openForSlot(slot);
    else requests.openForDay(previewDay);
  }

  async function send(): Promise<void> {
    if (!project) return;
    const title = project.charter?.title ?? 'Service';
    if (await requests.send({ title, creator: project.creator, settings, slots }))
      requests.listOpen = true;
  }

  async function saveSettings(): Promise<void> {
    if (await requests.saveSettings()) await detail.load();
  }

  async function added(ok: Promise<boolean>): Promise<void> {
    if (await ok) await activities.load();
  }
</script>

<section id="participation-activities" class="phase-surface activity-ui">
  <div class="service-ui">
    <p class="owner-note">
      {#if creator}
        You run this service. Only you and each person asking can see their request.
      {:else}
        Offered by <a href={resolve('/profile/[agent]', { agent: project?.creator ?? '' })}
          >{detail.names[project?.creator ?? ''] ?? 'the creator'}</a
        >. Requests stay private between you and them.
        {#if !me && !requestsOff}
          <a href={resolve('/onboarding')}>Sign in to request a time</a>.
        {/if}
      {/if}
    </p>
    {#if requestsOff}
      <div class="requests-off-banner" role="status">Not taking requests.</div>
    {/if}
    {#if notice}<p class="validation-copy" role="status">{notice}</p>{/if}
  </div>
  {#if usesCalendar}
    <ActivityCalendar
      activities={slotRows}
      selectedDay={previewDay}
      selectedActivity={requests.form.slot}
      canCreate={false}
      createActive={availability.open}
      toneOf={(a) => (a.rolesOpen ? 'green' : null)}
      onDay={(iso) => (previewDay = iso)}
      onActivity={requestSlot}
      onCreate={() => availability.openSheet(previewDay)}
    />
  {:else}
    <div class="service-ui empty-card helper-card">
      <p>
        This service is currently running through direct written requests instead of listed
        availability.
      </p>
    </div>
  {/if}

  <PhaseWorkToolbar>
    {#if creator && usesCalendar}
      <RoundPlusButton
        standout
        label="Add availability"
        ariaLabel="Add availability"
        onclick={() => availability.openSheet(previewDay)}
      />
    {/if}
    {#if creator || requests.shown.length > 0}
      <RoundPlusButton
        label={creator ? 'Requests' : 'Your requests'}
        ariaLabel="Open service requests"
        onclick={() => {
          requests.listOpen = true;
          void requests.load();
        }}
      />
    {/if}
    {#if creator}
      <RoundPlusButton
        label="Request settings"
        ariaLabel="Request service settings changes"
        onclick={() => requests.openSettings(settings)}
      />
    {/if}
    {#if canRequest}
      <RoundPlusButton
        standout
        label="Request service"
        ariaLabel="Request this service"
        onclick={openRequest}
      />
    {/if}
  </PhaseWorkToolbar>

  <RequestsSheet state={requests} reviewer={creator} {requestsOff} {highlighted} />
  <RequestComposer
    state={requests}
    scheduleRequired={settings.mode === 'calendar'}
    onSend={() => void send()}
  />
  {#if creator}
    <RequestSettingsSheet state={requests} current={settings} onSave={() => void saveSettings()} />
    <AvailabilitySheet
      state={availability}
      onOneOff={() => void added(availability.addOneOff(place))}
      onWeekly={() => void availability.addWeekly()}
    />
  {/if}

  <ServiceActivity
    {activities}
    {creator}
    {me}
    member={!!project?.amMember}
    {usesCalendar}
    {counts}
    nothingAhead={openSlots(slots) + accepted === 0}
    rules={availability.rules}
    {rows}
    chipOf={(a) => {
      const slot = slots.find((s) => s.id === a.id);
      return creator || !slot ? creatorChip(a, requestsOff) : slotChip(slot, requestsOff);
    }}
    onRemoveRule={(id) => void availability.remove(id)}
    onSelect={creator ? null : requestSlot}
  />
  <ServiceHistory {activities} {me} />
</section>
