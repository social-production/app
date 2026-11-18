<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import CopyKey from '#lib/components/CopyKey.svelte';
  import SettingPanel from '#lib/components/settings/SettingPanel.svelte';
  import { taggedName, type Person } from '#lib/domain/people.ts';
  import { explain } from '#lib/services/errors.ts';
  import {
    deviceRequests,
    devicesOf,
    getPerson,
    writeDeviceLink,
    type DeviceLink,
  } from '#lib/services/people.ts';

  let { me }: { me: string } = $props();

  let paired = $state<Person[]>([]);
  let requests = $state<{ link: DeviceLink; from: Person }[]>([]);
  let mainKey = $state('');
  let status = $state<string>();
  let error = $state<string>();
  let pairing = $state(false);

  const canSend = $derived(mainKey.trim().startsWith('uhCAk') && mainKey.trim() !== me);

  async function load(): Promise<void> {
    try {
      const [agents, links] = await Promise.all([devicesOf(me), deviceRequests()]);
      paired = await Promise.all(agents.map(getPerson));
      requests = await Promise.all(
        links.map(async (link) => ({
          link,
          from: await getPerson(link.device === me ? link.primary : link.device),
        })),
      );
    } catch (e) {
      error = explain(e);
    }
  }

  async function act(link: DeviceLink, done: string): Promise<void> {
    error = undefined;
    try {
      await writeDeviceLink(link);
      status = done;
      mainKey = '';
      await load();
    } catch (e) {
      error = explain(e);
    }
  }

  $effect(() => {
    void load();
  });
</script>

<section class="settings-section" id="settings-devices">
  <h2>{t('Devices')}</h2>
  <p class="section-intro">
    {t(
      'Each app install has its own key. Pair it with your main account so people see one person.',
    )}
  </p>
  <div class="card stack flush">
    <SettingPanel
      label={t('Paired devices')}
      value={paired.length ? paired.map((d) => d.profile?.name ?? 'Unnamed').join(', ') : t('None')}
    >
      {#each paired as device (device.agent)}
        <p class="setting-line">{taggedName(device)}</p>
      {:else}
        <p class="setting-hint">{t('No other devices paired yet.')}</p>
      {/each}
    </SettingPanel>

    {#each requests as request (request.link.device + request.link.primary)}
      <div class="set-row set-action">
        <span class="set-row-label">
          {t('Pairing request')}
          <small>From {taggedName(request.from)}</small>
        </span>
        <button class="set-pill" type="button" onclick={() => act(request.link, 'Device paired.')}
          >{t('Confirm')}</button
        >
      </div>
    {/each}

    <SettingPanel
      label={t('This device’s key')}
      value={me}
      key
      hint={t(
        'Paste it on your main account, or paste the main account’s key under “Pair with my main account”.',
      )}
    >
      <CopyKey value={me} label={t("Copy this device's key")} />
    </SettingPanel>

    <SettingPanel
      label={t('Pair with my main account')}
      value=""
      hint={t('Paste the key from “Copy my key” on your main account.')}
      bind:open={pairing}
    >
      <input
        bind:value={mainKey}
        aria-label={t('Main account key')}
        placeholder="uhCAk…"
        spellcheck="false"
      />
      {#snippet footer()}
        <div class="sheet-actions">
          <button
            class="sheet-submit"
            disabled={!canSend}
            type="button"
            onclick={() => {
              pairing = false;
              void act(
                { primary: mainKey.trim(), device: me },
                'Request sent. Confirm it on your main account.',
              );
            }}>{t('Send pairing request')}</button
          >
        </div>
      {/snippet}
    </SettingPanel>
  </div>
  <p class:profile-image-error={!!error} class:bio-saved={!error} class="status-line">
    {error ?? status ?? ''}
  </p>
</section>

<style>
  .status-line {
    min-height: 1.5em;
  }
</style>
