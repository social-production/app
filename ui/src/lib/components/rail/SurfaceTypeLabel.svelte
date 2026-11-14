<script lang="ts">
  import SurfaceIcon from '#lib/components/icons/SurfaceIcon.svelte';
  import { modeAccent, type CharterMode } from '#lib/domain/projects.ts';

  /** web's SurfaceTypeLabel for a project, an event or a help request. */
  let { projectMode, feedGlyph = false }: { projectMode: CharterMode; feedGlyph?: boolean } =
    $props();

  // web's projectSubjectLabel, and `surfaceTypeLabel('event')`.
  const LABELS: Record<CharterMode, string> = {
    proposal: 'Proposal',
    productive: 'Project',
    'collective-service': 'Service',
    'personal-service': 'Personal Service',
    event: 'Event',
    'help-request': 'Request',
    thread: 'Thread',
    post: 'Post',
    channel: 'Channel',
    community: 'Community',
  };
  const typeIcon = $derived(
    projectMode === 'event' ||
      projectMode === 'help-request' ||
      projectMode === 'channel' ||
      projectMode === 'thread' ||
      projectMode === 'post'
      ? projectMode
      : projectMode === 'community'
        ? 'community'
        : 'project',
  );
  // A feed row's glyph (owner, 2026-11-15): the mockups' cube for a project and speech
  // bubble for a thread, not web's house and lines.
  const icon = $derived(
    !feedGlyph
      ? typeIcon
      : typeIcon === 'project'
        ? 'cube'
        : typeIcon === 'thread'
          ? 'bubble'
          : typeIcon,
  );
</script>

<span class="surface-type-label" style={`--surface-accent: ${modeAccent(projectMode)}`}>
  <SurfaceIcon {icon} size="sm" />
  <span class="label-copy">{LABELS[projectMode]}</span>
</span>
