<script lang="ts">
  import { fitText } from '#lib/components/actions/fitText.ts';
  import type { PhaseStep } from '#lib/domain/projects.ts';

  import '#lib/components/styles/governance.css';

  /** web's DetailPhaseStepper: the project's phases, earlier ones complete, the current one marked. */
  interface Props {
    steps: PhaseStep[];
    current: string;
    selected: string;
    onSelect: (id: string) => void;
  }

  let { steps, current, selected, onSelect }: Props = $props();

  const currentIndex = $derived(steps.findIndex((s) => s.id === current));
</script>

<nav class="phase-bar overview-phase-tabs" aria-label="Phases">
  {#each steps as step, i (step.id)}
    {@const state = i < currentIndex ? 'complete' : i === currentIndex ? 'current' : 'future'}
    <button
      class="segment"
      class:active={selected === step.id}
      class:current={state === 'current'}
      class:complete={state === 'complete'}
      class:future={state === 'future'}
      type="button"
      aria-current={state === 'current' ? 'step' : undefined}
      aria-pressed={selected === step.id}
      aria-label={`${step.label} · ${state}`}
      title={`${step.label} · ${state}`}
      onclick={() => onSelect(step.id)}
    >
      <span class="label" use:fitText={{ min: 9, text: step.label }}>{step.label}</span>
    </button>
  {/each}
</nav>
