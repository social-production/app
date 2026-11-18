<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { displayZone } from '#lib/domain/zone.ts';
  import CreatePreview from '#lib/components/create/CreatePreview.svelte';
  import CreateSheet from '#lib/components/create/CreateSheet.svelte';
  import LocationIntent from '#lib/components/create/LocationIntent.svelte';
  import { searchFailure, searchPlaces } from '#lib/services/locations.ts';
  import PolicyNotice from '#lib/components/create/PolicyNotice.svelte';
  import RequiredLabel from '#lib/components/create/RequiredLabel.svelte';
  import { tagCounts } from '#lib/domain/scopes.ts';
  import { nextDay } from '#lib/domain/time.ts';
  import HelpRolesStep from './HelpRolesStep.svelte';
  import ScopeTagsStep from '../scopes/ScopeTagsStep.svelte';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  import { HELP_STEPS, HelpCreateState } from './helpCreateState.svelte.ts';

  /**
   * web's CreateHelpRequestPage: basics → location → timing → scope → roles → overview
   * (ADR 0067). Scope is the app's tags until channels and communities exist (M5d).
   */
  const s = new HelpCreateState();
  const zone = displayZone();

  async function publish(): Promise<void> {
    const id = await s.submit();
    if (id) await goto(resolve('/help-requests/[id]', { id }));
  }

  function setRole(index: number, patch: Partial<(typeof s.roles)[number]>): void {
    s.roles = s.roles.map((r, i) => (i === index ? { ...r, ...patch } : r));
  }

  // The rows edit `neededAt` / `endsAt` (local "YYYY-MM-DDTHH:MM"); an end before the start
  // is the next morning.
  function setDay(day: string): void {
    if (!day) return;
    const end = s.endsAt.slice(11, 16);
    s.setNeededAt(`${day}T${s.neededAt.slice(11, 16) || '09:00'}`);
    if (end) setEnd(end);
  }
  function setStart(time: string): void {
    if (time) s.setNeededAt(`${s.neededAt.slice(0, 10)}T${time}`);
  }
  function setEnd(time: string): void {
    if (!time) return;
    const day = s.neededAt.slice(0, 10);
    s.endsAt = time > s.neededAt.slice(11, 16) ? `${day}T${time}` : `${nextDay(day)}T${time}`;
  }
</script>

<CreateSheet
  title={t('Ask for help')}
  description={t(
    'A help request asks people to take a role for a specific need, with a time and a place.',
  )}
  steps={[...HELP_STEPS]}
  stepIndex={s.step}
  missing={s.missing}
  submitLabel={t('Publish help request')}
  submitting={s.submitting}
  onStep={(i) => (s.step = i)}
  onSubmit={() => void publish()}
  onClose={() => void goto(resolve('/'))}
>
  {#if s.stepId === 'basics'}
    <div class="form-stack">
      <label class="field">
        <RequiredLabel>{t('Title')}</RequiredLabel>
        <input
          bind:value={s.title}
          maxlength="200"
          aria-required="true"
          placeholder={t('e.g. Need a lift to the station')}
        />
      </label>
      <label class="field">
        <RequiredLabel>{t('What do you need?')}</RequiredLabel>
        <textarea
          bind:value={s.body}
          rows="4"
          aria-required="true"
          placeholder={t('Describe the one-off help needed.')}></textarea>
      </label>
      <PolicyNotice request />
    </div>
  {:else if s.stepId === 'location'}
    <div class="form-stack">
      <LocationIntent
        intent={s.intent}
        place={s.placeValue}
        search={searchPlaces}
        failure={searchFailure}
        allowLater={false}
        helperText={t('Where people should come, or that it happens online.')}
        onIntent={(i) => (s.intent = i)}
        onPlace={(p) => s.setPlace(p)}
      />
    </div>
  {:else if s.stepId === 'timing'}
    <!-- Polish Pass mockups: a date and two times as rows that open the phone's own pickers. -->
    <div class="form-stack time-rows">
      <label class="tap-row">
        <span>{t('Date')}</span>
        <input
          type="date"
          value={s.neededAt.slice(0, 10)}
          onchange={(e) => setDay(e.currentTarget.value)}
          aria-required="true"
        />
      </label>
      <div class="tap-pair">
        <label class="tap-row">
          <span>{t('Starts')}</span>
          <input
            type="time"
            aria-label={t('Start time')}
            value={s.neededAt.slice(11, 16)}
            onchange={(e) => setStart(e.currentTarget.value)}
          />
        </label>
        <label class="tap-row">
          <span>{t('Ends')}</span>
          <input
            type="time"
            aria-label={t('End time')}
            value={s.endsAt.slice(11, 16)}
            onchange={(e) => setEnd(e.currentTarget.value)}
          />
        </label>
      </div>
      <p class="zone-note">{zone} time. Everyone sees it in their own.</p>
    </div>
  {:else if s.stepId === 'scope'}
    <div class="form-stack">
      <ScopeTagsStep tags={s.tags} onChange={(t) => (s.tags = t)} variant="help" />
    </div>
  {:else if s.stepId === 'roles'}
    <HelpRolesStep
      roles={s.roles}
      onSet={setRole}
      onAdd={() => s.addRole()}
      onRemove={(i) => s.removeRole(i)}
    />
  {:else}
    <div class="form-stack overview">
      <CreatePreview
        mode="help-request"
        scope={s.tags[0]?.slice(1) ?? ''}
        title={s.title.trim()}
        body={s.body.trim()}
        when={s.schedule}
        place={s.intent === 'physical' ? s.locationLabel : ''}
      />
      <button class="overview-row" type="button" onclick={() => (s.step = 1)}>
        <strong>{t('Place')}</strong><span>{s.locationLabel}</span><em>{t('Edit')}</em>
      </button>
      <button class="overview-row" type="button" onclick={() => (s.step = 2)}>
        <strong>{t('When')}</strong><span>{s.schedule}</span><em>{t('Edit')}</em>
      </button>
      <button class="overview-row" type="button" onclick={() => (s.step = 3)}>
        <strong>{t('Channels')}</strong><span>{tagCounts(s.tags)}</span><em>{t('Edit')}</em>
      </button>
      <button class="overview-row" type="button" onclick={() => (s.step = 4)}>
        <strong>{t('Roles')}</strong>
        <span>{s.namedRoles.map((r) => r.label).join(', ') || 'None yet'}</span><em>{t('Edit')}</em>
      </button>
      {#if s.allProblems.length > 0}
        <ul class="status-message-list">
          {#each s.allProblems as problem (problem)}<li>{problem}</li>{/each}
        </ul>
      {:else if s.message}
        <p class="status-message">{s.message}</p>
      {/if}
    </div>
  {/if}
</CreateSheet>
