<script lang="ts">
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
    <h2>Audience</h2>
    <p class="description">Who can discover and access this event.</p>
    <div class="body">
      <label class="control-option">
        <input
          type="radio"
          name="event-audience"
          value="public"
          checked={audience === 'public'}
          onchange={() => onAudience('public')}
        />
        <span>
          <strong>Public</strong>
          <span class="hint"
            >Discoverable through channel/community tags. Always collaborative.</span
          >
        </span>
      </label>
      <label class="control-option">
        <input
          type="radio"
          name="event-audience"
          value="private_community"
          checked={audience === 'private_community'}
          onchange={() => onAudience('private_community')}
        />
        <span>
          <strong>Private community</strong>
          <span class="hint"
            >Only members of one private community can find it on feeds and the map.</span
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
          <strong>Invite only</strong>
          <span class="hint"
            >Only invited people and organizers can find it, including on the map.</span
          >
        </span>
      </label>
    </div>
  </section>
{:else if audience === 'public'}
  <section class="panel">
    <h2>Control</h2>
    <p class="description">Public events stay collaborative.</p>
    <div class="body">
      <p class="helper-text">
        Current mode: <strong>Collaborative</strong>. Members shape the plan through signals,
        values, and votes.
      </p>
    </div>
  </section>
{:else}
  <section class="panel">
    <h2>Control</h2>
    <p class="description">How this private event is run.</p>
    <div class="body">
      <p class="helper-text">
        {#if audience === 'private_community' && communityLabel}
          Private community: <strong>{communityLabel}</strong>.
        {:else if audience === 'invite_only'}
          Invite-only ({inviteeCount} invited).
        {:else}
          Private audience selected.
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
          <strong>Collaborative</strong>
          <span class="hint"
            >Same flow as public events: signals → values → plans → activity, just inside this
            private audience. Members can propose, vote, and sign up for roles.</span
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
          <strong>Organizer-controlled</strong>
          <span class="hint"
            >Skip signals, values, and proposal votes. Set the plan up front and start in Activity.
            Creators and co-organizers manage changes; members can still join and sign up for roles.</span
          >
        </span>
      </label>
      <p class="helper-text">
        Current selection: <strong
          >{governance === 'collaborative' ? 'Collaborative' : 'Organizer-controlled'}</strong
        >
      </p>
    </div>
  </section>
{/if}
