<script lang="ts">
  import SurfaceIcon from '#lib/components/icons/SurfaceIcon.svelte';
  import { modeAccent, type CharterMode } from '#lib/domain/projects.ts';

  /** web's SurfaceTypeLabel for a project, an event or a help request. */
  let { projectMode }: { projectMode: CharterMode } = $props();

  // web's projectSubjectLabel, and `surfaceTypeLabel('event')`.
  const LABELS: Record<CharterMode, string> = {
    productive: 'Productive',
    'collective-service': 'Collective Service',
    'personal-service': 'Personal Service',
    event: 'Event',
    'help-request': 'Help request',
    channel: 'Channel',
    community: 'Community',
  };
  const icon = $derived(
    projectMode === 'event' || projectMode === 'help-request' || projectMode === 'channel'
      ? projectMode
      : projectMode === 'community'
        ? 'community'
        : 'project',
  );
</script>

<span class="surface-type-label" style={`--surface-accent: ${modeAccent(projectMode)}`}>
  <SurfaceIcon {icon} size="sm" />
  <span class="label-copy">{LABELS[projectMode]}</span>
</span>
