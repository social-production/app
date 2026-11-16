<script lang="ts">
  import '#lib/components/styles/ux/loading.css';

  /** Owner's fix 3b: a thin brand line under the top bar while a page's data loads. Render it
   * beside a skeleton; several at once draw as one (they share the same place). */
</script>

<div class="loading-line" role="progressbar" aria-label="Loading"></div>
