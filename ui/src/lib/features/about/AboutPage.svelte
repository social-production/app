<script lang="ts">
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  import {
    ABOUT_SECTIONS,
    aboutBoard,
    aboutCosts,
    aboutJoin,
    aboutProduce,
    aboutRoadmap,
    aboutWhy,
    type AboutTabId,
  } from './aboutContent.ts';
  import {
    aboutGovernanceSummary,
    moderationAudienceSteps,
    moderationExamples,
    platformVoteExamples,
    projectVoteExamples,
    quorumFormulaSteps,
  } from './aboutVotes.ts';

  import '#lib/components/styles/about.css';

  /** web's AboutPage: the essay in tabs, the open tab in `?section=`. */
  const sectionIds = new Set<string>(ABOUT_SECTIONS.map((section) => section.id));

  function readSection(value: string | null): AboutTabId {
    return value && sectionIds.has(value) ? (value as AboutTabId) : 'purpose';
  }

  const active = $derived(readSection(page.url.searchParams.get('section')));

  function selectSection(id: AboutTabId): void {
    const next = new URL(page.url.href);
    if (id === 'purpose') next.searchParams.delete('section');
    else next.searchParams.set('section', id);
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- this page's own path
    void goto(`${next.pathname}${next.search}`, { replace: true, reset: false });
  }

  function formatCount(n: number): string {
    return n.toLocaleString('en-US');
  }
</script>

<div class="about-page">
  <section class="page">
    <header class="hero">
      <p class="eyebrow">About</p>
      <h1>Collective coordination of activity and production without exchange.</h1>
    </header>

    <div class="reader">
      <div class="tab-row" role="tablist" aria-label="About">
        {#each ABOUT_SECTIONS as section (section.id)}
          <button
            aria-selected={active === section.id}
            class:active-tab={active === section.id}
            class="detail-surface-tab tab"
            role="tab"
            type="button"
            onclick={() => selectSection(section.id)}
          >
            <span class="tab-label-full">{section.label}</span>
            <span class="tab-label-compact">
              {section.id === 'governance'
                ? 'Gov'
                : section.id === 'moderation'
                  ? 'Mod'
                  : section.label}
            </span>
          </button>
        {/each}
      </div>

      <article class="essay">
        {#if active === 'purpose'}
          <h2>Purpose</h2>
          <p>{aboutWhy.lead}</p>
          <p class="definition-intro">
            The key to understanding what Social Production is trying to do is understanding the
            difference between use-value and exchange-value.
          </p>
          {#each aboutWhy.framing as item (item.title)}
            <section class="definition">
              <p class="definition-label">Definition</p>
              <h3>{item.title}</h3>
              <p>{item.body}</p>
            </section>
          {/each}
          {#each aboutWhy.points as point (point)}
            <p>{point}</p>
          {/each}
          <h3>Further reading</h3>
          {#each aboutWhy.reading as link (link.href)}
            <p>
              <a href={link.href} rel="external noreferrer" target="_blank">{link.label}</a>
              {link.blurb}
            </p>
          {/each}
        {:else if active === 'strategy'}
          <h2>Strategy</h2>
          <p>{aboutProduce.lead}</p>
          {#each aboutProduce.steps as step, index (step.title)}
            <h3>{index + 1}. {step.title}</h3>
            <p>{step.body}</p>
          {/each}
          <p>{aboutProduce.note}</p>
        {:else if active === 'roadmap'}
          <h2>Roadmap</h2>
          <p>
            The prototype comes first. A legal home for funds comes next. Peer-to-peer
            infrastructure is later. The aim is coordination that is hard to capture or switch off.
          </p>
          {#each aboutRoadmap as phase (phase.title)}
            <div class="phase-heading">
              <span class:current={phase.current} class="phase-tag">{phase.label}</span>
              <h3>{phase.title}</h3>
            </div>
            {#each phase.points as point (point)}
              <p>{point}</p>
            {/each}
          {/each}
        {:else if active === 'costs'}
          <h2>Costs and ownership</h2>
          <p>{aboutCosts.lead}</p>
          {#each aboutCosts.cards as card (card.title)}
            <div class="phase-heading">
              <span class:current={card.title === 'Phase 1'} class="phase-tag">{card.title}</span>
            </div>
            <p>{card.body}</p>
          {/each}
        {:else if active === 'governance'}
          <h2>Governance</h2>
          <p>{aboutGovernanceSummary.lead}</p>
          {#each aboutGovernanceSummary.rules as rule (rule)}
            <p>{rule}</p>
          {/each}
          <h3>How quorum is derived</h3>
          {#each quorumFormulaSteps as step, index (step)}
            <p>{index + 1}. {step}</p>
          {/each}
          <h3>Project votes</h3>
          {#each projectVoteExamples as example (example.title)}
            <p>
              <strong>{example.title}.</strong>
              {example.audience}. N = {formatCount(example.n)}, quorum {formatCount(
                example.required,
              )} ({example.percent}).
              {example.note}
            </p>
          {/each}
          <h3>Platform votes</h3>
          <p>
            Platform-tagged projects and events affect everyone, so quorum is sized from platform
            weekly actives. That still applies when other channels or communities are also tagged.
          </p>
          {#each platformVoteExamples as example (example.title)}
            <p>
              <strong>{example.title}.</strong>
              N = {formatCount(example.n)}, quorum {formatCount(example.required)} ({example.percent}).
              {example.note}
            </p>
          {/each}
          <h3>Board</h3>
          <p>{aboutBoard.lead}</p>
          {#each aboutBoard.points as point (point)}
            <p>{point}</p>
          {/each}
        {:else if active === 'moderation'}
          <h2>Moderation</h2>
          <p>
            Moderation uses the same audience-derived quorum and a yes share of at least 66% among
            votes cast. Serious harm can blur and hide sooner, then remove at a higher bar. Spam
            uses the full delete quorum. Older or highly engaged content can raise the required yes
            share above 66%.
          </p>
          <h3>Where the audience comes from</h3>
          {#each moderationAudienceSteps as step (step)}
            <p>{step}</p>
          {/each}
          {#each moderationExamples as example (example.title)}
            <p>
              <strong>{example.title}.</strong>
              {example.audience}. N = {formatCount(example.n)}.
              {#if example.reason === 'spam'}
                Delete quorum {formatCount(example.deleteQuorum)}.
              {:else}
                Hide quorum {formatCount(example.hideQuorum ?? 0)}, delete quorum {formatCount(
                  example.deleteQuorum,
                )}.
              {/if}
              Yes share floor {example.yesShare}. {example.note}
            </p>
          {/each}
        {:else}
          <h2>Join</h2>
          <p>{aboutJoin.lead}</p>
          {#each aboutJoin.recap as point (point)}
            <p>{point}</p>
          {/each}
          <h3>Community</h3>
          {#each aboutJoin.links as link (link.href)}
            <p><a href={link.href} rel="external noreferrer" target="_blank">{link.label}</a></p>
          {/each}
        {/if}

        <a class="discover" href={resolve('/')}>Get involved!</a>
      </article>
    </div>
  </section>
</div>
