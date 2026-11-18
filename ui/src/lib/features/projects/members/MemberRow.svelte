<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';

  import { resolve } from '$app/paths';

  /**
   * One row of the member list (Detail Pages Cleanup §6): face, name, and a second line
   * (where they live and their standing, or who invited them and when). Your own row says
   * You. ⋯ holds View profile and Message, and an organiser's Make organiser / Remove.
   */
  interface Action {
    label: string;
    icon: string;
    danger?: boolean;
    run: () => void;
  }

  interface Props {
    agent: string;
    name: string;
    line: string;
    you?: boolean;
    badge?: string;
    actions?: Action[];
    busy?: boolean;
  }

  let { agent, name, line, you = false, badge = '', actions = [], busy = false }: Props = $props();
  let open = $state(false);
</script>

<div class="member-row" class:you>
  <AvatarBadge size="md" {name} />
  <a class="member-copy" href={resolve('/profile/[agent]', { agent })}>
    <strong>{you ? t('You') : name}</strong>
    {#if line}<small>{line}</small>{/if}
  </a>
  {#if badge}<span class="member-badge">{badge}</span>{/if}
  {#if !you}
    <span class="member-more">
      <button
        type="button"
        class="member-dots"
        aria-label={`More for ${name}`}
        aria-haspopup="menu"
        aria-expanded={open}
        disabled={busy}
        onclick={() => (open = !open)}><ToolbarIcon name="kebab" /></button
      >
      {#if open}
        <button
          class="member-backdrop"
          type="button"
          aria-label={t('Close menu')}
          onclick={() => (open = false)}
        ></button>
        <div class="member-menu" role="menu">
          <a role="menuitem" href={resolve('/profile/[agent]', { agent })}
            ><ToolbarIcon name="user" />{t('View profile')}</a
          >
          <a
            role="menuitem"
            href={resolve(`/messages?to=${encodeURIComponent(agent)}` as '/messages')}
            ><ToolbarIcon name="send" />{t('Message')}</a
          >
          {#each actions as action (action.label)}
            <button
              type="button"
              role="menuitem"
              class:danger={action.danger}
              onclick={() => {
                open = false;
                action.run();
              }}><ToolbarIcon name={action.icon} />{action.label}</button
            >
          {/each}
        </div>
      {/if}
    </span>
  {/if}
</div>
