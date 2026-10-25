<script lang="ts">
  import type { FollowRow } from '#lib/domain/follows.ts';
  import { explain } from '#lib/services/errors.ts';
  import {
    decideFollow,
    followRequests,
    followView,
    setFollowApproval,
  } from '#lib/services/follows.ts';
  import { namesOf } from '#lib/services/people.ts';

  /** web's "Follow requests" and "Privacy" settings. The directory is public to its network,
   *  so approval decides whose follow counts; it can't make anything secret (ADR 0054). */
  /** `onRequests`: how many requests wait (the settings nav links them when there are any). */
  let { me, onRequests }: { me: string; onRequests: (count: number) => void } = $props();

  let requests = $state<FollowRow[]>([]);
  let names = $state<Record<string, string>>({});
  let approval = $state(false);
  let pending = $state<string | null>(null);
  let error = $state<string>();

  async function load(): Promise<void> {
    try {
      const [view, waiting] = await Promise.all([followView(me), followRequests()]);
      approval = view.requiresApproval;
      requests = waiting;
      onRequests(waiting.length);
      names = await namesOf(waiting.map((r) => r.agent));
    } catch (e) {
      error = explain(e);
    }
  }

  async function run(key: string, action: () => Promise<void>): Promise<void> {
    pending = key;
    error = undefined;
    try {
      await action();
      await load();
    } catch (e) {
      error = explain(e);
    } finally {
      pending = null;
    }
  }

  $effect(() => {
    void load();
  });
</script>

{#if requests.length > 0}
  <section class="settings-section" id="settings-follow-requests">
    <h2>Follow requests</h2>
    <div class="card stack">
      {#each requests as request (request.agent)}
        <div class="follow-request-row">
          <div><strong>@{names[request.agent] ?? 'Unnamed'}</strong></div>
          <div class="follow-request-actions">
            <button
              class="button-primary"
              disabled={pending === request.agent}
              type="button"
              onclick={() => run(request.agent, () => decideFollow(request.agent, true))}
            >
              Accept
            </button>
            <button
              class="button-ghost"
              disabled={pending === request.agent}
              type="button"
              onclick={() => run(request.agent, () => decideFollow(request.agent, false))}
            >
              Decline
            </button>
          </div>
        </div>
      {/each}
    </div>
  </section>
{/if}

<section class="settings-section" id="settings-privacy">
  <h2>Privacy</h2>
  <p class="section-intro">
    Your profile and follows are public to everyone on the network. Approval decides whose follow
    counts and who appears as your follower.
  </p>
  <div class="card stack flush">
    <div class="setting-item">
      <div>
        <strong>Require approval to follow you</strong>
        <p>When on, new followers must be approved before they can see follower-only content.</p>
      </div>
      <button
        aria-checked={approval}
        aria-label="Require approval to follow you"
        class="switch"
        class:on={approval}
        disabled={pending === 'approval'}
        role="switch"
        type="button"
        onclick={() => run('approval', () => setFollowApproval(!approval))}
      >
        <span class="switch-thumb"></span>
        <span class="sr-only">{approval ? 'On' : 'Off'}</span>
      </button>
    </div>
  </div>
  {#if error}<p class="warning-card" role="alert">{error}</p>{/if}
</section>
