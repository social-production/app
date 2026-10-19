<script lang="ts">
  import ContentMetaRow from '#lib/components/cards/ContentMetaRow.svelte';
  import FeedCardTop from '#lib/components/cards/FeedCardTop.svelte';
  import UpdatesSection from '#lib/components/governance/UpdatesSection.svelte';
  import type { Charter, Project } from '#lib/domain/projects.ts';

  import { resolve } from '$app/paths';

  /** web's ProjectOverviewHeader context panel, with the updates section below it. */
  interface Props {
    project: Project;
    charter: Charter;
    names: Record<string, string>;
    canPropose: boolean;
    onAddUpdate: () => void;
  }

  let { project, charter, names, canPropose, onAddUpdate }: Props = $props();
</script>

<div class="project-overview">
  <div class="context-panel">
    <FeedCardTop tags={charter.tags} stage={project.stage} />
    <div class="heading overview-heading">
      <div class="identity-row">
        <div class="identity-copy">
          <h1>{charter.title}</h1>
          {#if charter.location_label}<p class="live-fact">{charter.location_label}</p>{/if}
          <p class="overview-copy">{charter.description}</p>
          {#if project.updates[0]}<p class="overview-update">
              Update: {project.updates[0].body}
            </p>{/if}
        </div>
      </div>
    </div>
    <div class="context-meta">
      <ContentMetaRow
        fullName
        authorName={names[project.creator] ?? '…'}
        authorHref={resolve('/profile/[agent]', { agent: project.creator })}
        createdAt={project.createdAt}
      />
    </div>
  </div>
</div>
<UpdatesSection updates={project.updates} {names} {canPropose} onAdd={onAddUpdate} />
