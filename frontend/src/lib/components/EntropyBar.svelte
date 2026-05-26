<script lang="ts">
  export let score: number; // 0-4 from zxcvbn

  const labels = ["Débil", "Regular", "Buena", "Fuerte", "Excelente"];
  const colors = ["var(--danger)", "#e08a4d", "#d4c953", "#7bc94a", "var(--success)"];

  $: safeScore = Math.max(0, Math.min(4, score));
  $: label = labels[safeScore];
  $: color = colors[safeScore];
</script>

<div class="entropy-bar" role="progressbar" aria-valuenow={safeScore} aria-valuemin={0} aria-valuemax={4} aria-label={`Fortaleza: ${label}`}>
  <div class="segments">
    {#each [0, 1, 2, 3, 4] as i}
      <div
        class="segment"
        class:active={i <= safeScore}
        style:background-color={i <= safeScore ? color : "var(--bg-tertiary)"}
      ></div>
    {/each}
  </div>
  <span class="label" style:color>{label}</span>
</div>

<style>
  .entropy-bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .segments {
    display: flex;
    gap: 3px;
    flex: 1;
  }

  .segment {
    flex: 1;
    height: 6px;
    border-radius: 2px;
    transition: background-color 0.2s ease;
  }

  .label {
    font-size: 0.75rem;
    font-weight: 500;
    min-width: 5rem;
    text-align: right;
  }
</style>
