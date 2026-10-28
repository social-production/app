<script lang="ts">
  import CreateSheet from '#lib/components/create/CreateSheet.svelte';
  import EventAudiencePanel from '#lib/components/create/EventAudiencePanel.svelte';
  import EventPlanStep from '#lib/components/create/EventPlanStep.svelte';
  import LocationIntent from '#lib/components/create/LocationIntent.svelte';
  import PeoplePicker from '#lib/components/create/PeoplePicker.svelte';
  import PolicyNotice from '#lib/components/create/PolicyNotice.svelte';
  import RequiredLabel from '#lib/components/create/RequiredLabel.svelte';
  import TagSelector from '#lib/components/create/TagSelector.svelte';
  import { EventCreateState } from './eventCreateState.svelte.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  /**
   * web's CreateEventPage: basics → audience → tags (public) or who + control (private) →
   * location → plan (organizer-controlled) → overview. A public event starts as a proposal;
   * an organizer-controlled one starts in Activity with its plan (ADR 0057, 0061).
   */
  const s = new EventCreateState();

  const organizerHelp = $derived(
    s.upfrontPlan
      ? 'You keep full organizer authority. Chosen organizers can invite people, create activities, and manage event decisions; other invitees can only attend and sign up.'
      : 'You keep full organizer authority. Chosen organizers can invite people; other invitees can create activities, plans, and proposals, but only organizers can invite.',
  );
  const missing = $derived(
    s.canContinue ? s.missing : [...s.missing, s.stepId === 'plan' ? 'plan details' : 'invites'],
  );

  async function publish(): Promise<void> {
    const id = await s.submit();
    if (id) await goto(resolve('/events/[id]', { id }));
  }
</script>

<CreateSheet
  title="Create event"
  description="An event gathers people for a planned occasion. A public event starts as a proposal, and its dates are set when members approve its plan. A private event can get its dates now."
  steps={s.steps}
  stepIndex={s.step}
  {missing}
  submitLabel="Publish event"
  submitting={s.submitting}
  onStep={(i) => (s.step = i)}
  onSubmit={() => void publish()}
  onClose={() => void goto(resolve('/'))}
>
  {#if s.stepId === 'basics'}
    <div class="form-stack">
      <label>
        <RequiredLabel>Event title</RequiredLabel>
        <input bind:value={s.title} maxlength="200" aria-required="true" />
      </label>
      <label>
        <RequiredLabel>Description</RequiredLabel>
        <textarea bind:value={s.description} rows="4" aria-required="true"></textarea>
      </label>
      <PolicyNotice event />
    </div>
  {:else if s.stepId === 'audience' || s.stepId === 'control'}
    <EventAudiencePanel
      mode={s.stepId}
      audience={s.audience}
      governance={s.governance}
      inviteeCount={s.invitees.length}
      onAudience={(a) => (s.audience = a)}
      onGovernance={(g) => (s.chosenGovernance = g)}
    />
  {:else if s.stepId === 'scope' && !s.isPrivate}
    <div class="form-stack">
      <TagSelector label="Tags" tags={s.tags} onChange={(t) => (s.tags = t)} />
    </div>
  {:else if s.stepId === 'scope'}
    <div class="form-stack">
      <PeoplePicker
        label="Invite people"
        placeholder="Type to add people"
        helperText="Invitees can join and take part once the event is underway."
        selected={s.invitees}
        search={(q) => s.search(q)}
        onAdd={(c) => s.addInvitee(c)}
        onRemove={(a) => (s.invitees = s.invitees.filter((c) => c.agent !== a))}
      />
      <PeoplePicker
        label="Organizers (optional)"
        placeholder="People with organizer authority"
        helperText={organizerHelp}
        selected={s.organizers}
        search={(q) => s.search(q)}
        onAdd={(c) => s.addOrganizer(c)}
        onRemove={(a) => (s.organizers = s.organizers.filter((c) => c.agent !== a))}
      />
      {#if s.invitees.length === 0}
        <p class="status-note">Invite at least one person for invite-only events.</p>
      {/if}
    </div>
  {:else if s.stepId === 'location'}
    <LocationIntent
      intent={s.intent}
      place={s.place}
      onIntent={(i) => (s.intent = i)}
      onPlace={(p) => (s.place = p)}
    />
  {:else if s.stepId === 'plan'}
    <EventPlanStep bind:plan={s.plan} problems={s.planProblems} />
  {:else}
    <div class="form-stack overview">
      <button class="overview-row" type="button" onclick={() => (s.step = s.stepOf('basics'))}>
        <strong>Basics</strong><span>{s.title.trim() || 'Untitled event'}</span>
      </button>
      <button class="overview-row" type="button" onclick={() => (s.step = s.stepOf('audience'))}>
        <strong>Audience</strong><span>{s.isPrivate ? 'Invite only' : 'Public'}</span>
      </button>
      <button class="overview-row" type="button" onclick={() => (s.step = s.stepOf('scope'))}>
        <strong>{s.isPrivate ? 'Who' : 'Tags'}</strong>
        <span>
          {#if s.isPrivate}
            {s.invitees.length} invites{#if s.organizers.length}
              · {s.organizers.length} organizers{/if}
          {:else}
            {s.tags.length} {s.tags.length === 1 ? 'tag' : 'tags'}
          {/if}
        </span>
      </button>
      {#if s.isPrivate}
        <button class="overview-row" type="button" onclick={() => (s.step = s.stepOf('control'))}>
          <strong>Control</strong>
          <span>{s.governance === 'collaborative' ? 'Collaborative' : 'Organizer-controlled'}</span>
        </button>
      {/if}
      <button class="overview-row" type="button" onclick={() => (s.step = s.stepOf('location'))}>
        <strong>Location</strong><span>{s.locationLabel}</span>
      </button>
      {#if s.upfrontPlan}
        <button class="overview-row" type="button" onclick={() => (s.step = s.stepOf('plan'))}>
          <strong>Plan</strong>
          <span
            >{s.plan.title.trim() || 'Untitled plan'} · {s.plan.stages.length}
            {s.plan.stages.length === 1 ? 'stage' : 'stages'}</span
          >
        </button>
      {:else}
        <div class="overview-row static">
          <strong>Dates</strong>
          <span>Set when members approve the event's plan</span>
        </div>
      {/if}
      {#if s.message}<p class="status-note">{s.message}</p>{/if}
    </div>
  {/if}
</CreateSheet>
