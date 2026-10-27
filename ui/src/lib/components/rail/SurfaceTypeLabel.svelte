<script lang="ts">
  import SurfaceIcon from '#lib/components/icons/SurfaceIcon.svelte';
  import { modeAccent, type CharterMode } from '#lib/domain/projects.ts';

  /** web's SurfaceTypeLabel for a project or an event (help requests come later in M5c). */
  let { projectMode }: { projectMode: CharterMode } = $props();

  // web's projectSubjectLabel, and `surfaceTypeLabel('event')`.
  const LABELS: Record<CharterMode, string> = {
    productive: 'Productive',
    'collective-service': 'Collective Service',
    'personal-service': 'Personal Service',
    event: 'Event',
  };
</script>

<span class="surface-type-label" style={`--surface-accent: ${modeAccent(projectMode)}`}>
  <SurfaceIcon icon={projectMode === 'event' ? 'event' : 'project'} size="sm" />
  <span class="label-copy">{LABELS[projectMode]}</span>
</span>
