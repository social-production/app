<script lang="ts">
  import SettingSwitch from '#lib/components/settings/SettingSwitch.svelte';
  import { loadPush, push, setPushDetails, setPushOn } from '#lib/stores/push.svelte.ts';

  /** Push notifications on this device (ADR 0108), under the categories they follow. */
  $effect(() => {
    void loadPush();
  });

  const pushNote = $derived(
    push.where === 'browser'
      ? 'Get the categories above on this device, even when the app is closed.'
      : 'This browser can’t get notifications. On an iPhone or iPad, add Social Production to your Home Screen and open it from there.',
  );
</script>

{#if push.where !== 'desktop'}
  <h3 class="settings-subhead">On this device</h3>
  <div class="card stack flush">
    <SettingSwitch
      label="Push notifications"
      note={pushNote}
      checked={push.on}
      disabled={push.busy || push.where !== 'browser'}
      onToggle={() => void setPushOn(!push.on)}
    />
    <SettingSwitch
      label="Show what it’s about"
      note="Off: a notification only says something is new, so your lock screen stays private."
      checked={push.details}
      disabled={push.busy || !push.on}
      onToggle={() => void setPushDetails(!push.details)}
    />
  </div>
  {#if push.problem}<p class="section-intro push-problem" role="alert">{push.problem}</p>{/if}
{/if}
