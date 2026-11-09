<script lang="ts">
  import HistoryTab from '#lib/components/governance/HistoryTab.svelte';
  import type { Proposal, VoteChoice } from '#lib/domain/governance.ts';
  import { linkHistory, linkRequestsById } from '#lib/domain/linkHistory.ts';
  import type { PlanView } from '#lib/domain/plans.ts';
  import type { Charter, Project } from '#lib/domain/projects.ts';

  import type { DetailState } from './detailState.svelte.ts';
  import { LinksState } from './linksState.svelte.ts';

  /**
   * The History tab of a project or event: its decisions, a software project's votes
   * (ADR 0086) and, as on web, its link create and sever votes (read when the tab opens, as
   * the Links tab reads them). A link vote is cast on this project's half.
   */
  interface Props {
    detail: DetailState;
    project: Project;
    charter: Charter;
    onOpenPlan: (plan: PlanView) => void;
  }

  let { detail, project, charter, onOpenPlan }: Props = $props();

  const links = new LinksState(() => detail.project);

  $effect(() => {
    void links.load();
  });

  const requests = $derived(linkRequestsById(links.frame));
  const extra = $derived([...detail.software.history, ...linkHistory(links.frame)]);

  function vote(p: Proposal, choice: VoteChoice): Promise<void> {
    const request = requests.get(p.id);
    return request ? links.vote(request.id, request.thisSide, choice) : detail.vote(p, choice);
  }
</script>

<HistoryTab
  proposals={detail.proposals}
  plans={detail.plans?.plans}
  names={{ ...detail.names, ...detail.software.names, ...links.names }}
  {extra}
  {charter}
  stage={project.stage}
  phaseLabel={detail.phaseLabel}
  onVote={vote}
  {onOpenPlan}
/>
