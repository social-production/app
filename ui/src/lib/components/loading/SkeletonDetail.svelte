<script lang="ts">
  import '#lib/components/styles/ux/loading.css';

  /** Owner's fix 3b: a detail page's header and facts card while it loads. */
</script>

<div class="skeleton-detail" aria-hidden="true">
  <div class="skeleton-head">
    <span class="sk sk-face"></span><span class="sk sk-line" style="width: 42%"></span>
  </div>
  <span class="sk sk-title big" style="width: 78%"></span>
  <span class="sk sk-line"></span>
  <span class="sk sk-line" style="width: 70%"></span>
  <div class="skeleton-facts">
    {#each { length: 3 }, i (i)}
      <div class="skeleton-fact">
        <span class="sk sk-icon"></span>
        <div>
          <span class="sk sk-line" style={`width: ${60 - i * 10}%`}></span>
          <span class="sk sk-line" style="width: 40%"></span>
        </div>
      </div>
    {/each}
  </div>
</div>
