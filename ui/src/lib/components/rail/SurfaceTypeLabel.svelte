<script lang="ts">
  import SurfaceIcon from '#lib/components/icons/SurfaceIcon.svelte';
  import { modeAccent, type ProjectMode } from '#lib/domain/projects.ts';

  /** web's SurfaceTypeLabel for a project (events and help requests come with M5c). */
  let { projectMode }: { projectMode: ProjectMode } = $props();

  // web's projectSubjectLabel.
  const LABELS: Record<ProjectMode, string> = {
    productive: 'Productive',
    'collective-service': 'Collective Service',
    'personal-service': 'Personal Service',
  };
</script>

<span class="surface-type-label" style={`--surface-accent: ${modeAccent(projectMode)}`}>
  <SurfaceIcon icon="project" size="sm" />
  <span class="label-copy">{LABELS[projectMode]}</span>
</span>
