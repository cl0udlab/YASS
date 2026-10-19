<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    onclose,
    maxWidth = "640px",
    children,
  }: { onclose?: () => void; maxWidth?: string; children: Snippet } = $props();
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onclose?.()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div
  class="modal-backdrop"
  onclick={(e) => e.target === e.currentTarget && onclose?.()}
>
  <div class="modal" role="dialog" aria-modal="true" style:max-width={maxWidth}>
    {@render children()}
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    z-index: 1000;
  }

  .modal {
    background-color: var(--card-bg);
    color: var(--text-color);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    width: 100%;
    max-height: 90vh;
    overflow-y: auto;
    padding: 1.5rem;
  }

  .modal :global(h3) {
    margin-top: 0;
  }

  .modal :global(.modal-actions) {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
    margin-top: 1.25rem;
    flex-wrap: wrap;
  }
</style>
