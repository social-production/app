<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import type { EventAudience, EventGovernance } from '#lib/domain/projects.ts';

  /**
   * web's CreateEventVisibilityPanel: the Audience step, or the Control step for a private
   * event (ADR 0061, 0069).
   */
  interface Props {
    mode: 'audience' | 'control';
    audience: EventAudience;
    governance: EventGovernance;
    inviteeCount: number;
    /** The chosen private community's handle. */
    communityLabel?: string | null;
    onAudience: (audience: EventAudience) => void;
    onGovernance: (governance: EventGovernance) => void;
  }

  let {
    mode,
    audience,
    governance,
    inviteeCount,
    communityLabel = null,
    onAudience,
    onGovernance,
  }: Props = $props();
</script>

{#if mode === 'audience'}
  <section class="panel">
    <h2>{t('Audience')}</h2>
    <p class="description">{t('Who can discover and access this event.')}</p>
    <div class="body">
      <label class="control-option">
        <input
          type="radio"
          name="event-audience"
          value="private_community"
          checked={audience === 'private_community'}
          onchange={() => onAudience('private_community')}
        />
        <span>
          <strong>{t('Private community')}</strong>
          <span class="hint"
            >{t('Only members of one private community can find it on feeds and the map.')}</span
          >
        </span>
      </label>
      <label class="control-option">
        <input
          type="radio"
          name="event-audience"
          value="invite_only"
          checked={audience === 'invite_only'}
          onchange={() => onAudience('invite_only')}
        />
        <span>
          <strong>{t('Invite only')}</strong>
          <span class="hint"
            >{t('Only invited people and organizers can find it, including on the map.')}</span
          >
        </span>
      </label>
    </div>
  </section>
{:else if audience === 'public'}
  <section class="panel">
    <h2>{t('Control')}</h2>
    <p class="description">{t('Public events stay collaborative.')}</p>
    <div class="body">
      <p class="helper-text">
        {t('Current mode:')} <strong>{t('Collaborative')}</strong>{t(
          '. Members shape the plan through signals, values, and votes.',
        )}
      </p>
    </div>
  </section>
{:else}
  <section class="panel">
    <h2>{t('Control')}</h2>
    <p class="description">{t('How this private event is run.')}</p>
    <div class="body">
      <p class="helper-text">
        {#if audience === 'private_community' && communityLabel}
          {t('Private community:')} <strong>{communityLabel}</strong>.
        {:else if audience === 'invite_only'}
          Invite-only ({inviteeCount} invited).
        {:else}
          {t('Private audience selected.')}
        {/if}
      </p>
      <label class="control-option">
        <input
          type="radio"
          name="event-governance"
          value="collaborative"
          checked={governance === 'collaborative'}
          onchange={() => onGovernance('collaborative')}
        />
        <span>
          <strong>{t('Collaborative')}</strong>
          <span class="hint"
            >{t(
              'Same flow as public events: signals → values → plans → activity, just inside this private audience. Members can propose, vote, and sign up for roles.',
            )}</span
          >
        </span>
      </label>
      <label class="control-option">
        <input
          type="radio"
          name="event-governance"
          value="organizer_controlled"
          checked={governance === 'organizer_controlled'}
          onchange={() => onGovernance('organizer_controlled')}
        />
        <span>
          <strong>{t('Organizer-controlled')}</strong>
          <span class="hint"
            >{t(
              'Skip signals, values, and proposal votes. Set the plan up front and start in Activity. Creators and co-organizers manage changes; members can still join and sign up for roles.',
            )}</span
          >
        </span>
      </label>
      <p class="helper-text">
        {t('Current selection:')}
        <strong
          >{governance === 'collaborative' ? t('Collaborative') : t('Organizer-controlled')}</strong
        >
      </p>
    </div>
  </section>
{/if}
