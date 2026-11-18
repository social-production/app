<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import SettingSwitch from '#lib/components/settings/SettingSwitch.svelte';
  import type { FollowRow } from '#lib/domain/follows.ts';
  import { explain } from '#lib/services/errors.ts';
  import {
    decideFollow,
    followRequests,
    followView,
    setFollowApproval,
    setPrivacy,
  } from '#lib/services/follows.ts';
  import { PRIVACY_SWITCHES, type PrivacySwitch } from '#lib/domain/privacy.ts';
  import { namesOf } from '#lib/services/people.ts';

  /** web's "Follow requests" and "Privacy" settings. The directory is public to its network,
   *  so approval decides whose follow counts; it can't make anything secret (ADR 0054). */
  /** `onRequests`: how many requests wait (the settings nav links them when there are any). */
  let { me, onRequests }: { me: string; onRequests: (count: number) => void } = $props();

  let requests = $state<FollowRow[]>([]);
  let names = $state<Record<string, string>>({});
  let approval = $state(false);
  let privacy = $state<PrivacySwitch[]>([]);
  let pending = $state<string | null>(null);
  let error = $state<string>();

  async function load(): Promise<void> {
    try {
      const [view, waiting] = await Promise.all([followView(me), followRequests()]);
      approval = view.requiresApproval;
      privacy = view.privacy;
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
    <h2>{t('Follow requests')}</h2>
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
              {t('Accept')}
            </button>
            <button
              class="button-ghost"
              disabled={pending === request.agent}
              type="button"
              onclick={() => run(request.agent, () => decideFollow(request.agent, false))}
            >
              {t('Decline')}
            </button>
          </div>
        </div>
      {/each}
    </div>
  </section>
{/if}

<section class="settings-section" id="settings-privacy">
  <h2>{t('Privacy')}</h2>
  <p class="section-intro">
    {t(
      "Your profile and follows are public to everyone on the network. Approval decides whose follow counts; the hide switches decide what the app shows to whom. Everyone on the network can still reach it; the app just doesn't show it.",
    )}
  </p>
  <div class="card stack flush">
    <SettingSwitch
      label={t('Require approval to follow you')}
      note={t('New followers must be approved before they see follower-only content.')}
      checked={approval}
      disabled={pending === 'approval'}
      onToggle={() => void run('approval', () => setFollowApproval(!approval))}
    />
    {#each PRIVACY_SWITCHES as item (item.name)}
      {@const on = privacy.includes(item.name)}
      <SettingSwitch
        label={item.label}
        note={item.note}
        checked={on}
        disabled={pending === item.name}
        onToggle={() => void run(item.name, () => setPrivacy(item.name, !on).then(() => undefined))}
      />
    {/each}
  </div>
  {#if error}<p class="warning-card" role="alert">{error}</p>{/if}
</section>
