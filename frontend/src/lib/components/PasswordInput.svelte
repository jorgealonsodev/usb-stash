<script lang="ts">
  export let value = "";
  export let id = "password";
  export let placeholder = "";
  export let disabled = false;

  let visible = false;

  function toggle() {
    visible = !visible;
  }

  function onInput(event: Event) {
    const target = event.target as HTMLInputElement;
    value = target.value;
  }
</script>

<div class="password-input">
  <!-- Hidden password input (always bound to value) -->
  <input
    type="password"
    {id}
    {placeholder}
    {disabled}
    bind:value
    class:hidden={visible}
    class="input-field"
  />
  <!-- Visible text input (shown when toggled, syncs value) -->
  <input
    type="text"
    {id}
    {placeholder}
    {disabled}
    value={value}
    on:input={onInput}
    class:hidden={!visible}
    class="input-field"
  />
  <button
    type="button"
    class="toggle"
    on:click={toggle}
    {disabled}
    aria-label={visible ? "Ocultar contraseña" : "Mostrar contraseña"}
  >
    {visible ? "◑" : "●"}
  </button>
</div>

<style>
  .password-input {
    display: flex;
    align-items: center;
    position: relative;
    width: 100%;
  }

  .input-field {
    flex: 1;
    padding-right: 2.5rem;
  }

  .input-field.hidden {
    display: none;
  }

  .toggle {
    position: absolute;
    right: 0.5rem;
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 1rem;
    padding: 0.25rem;
    cursor: pointer;
    line-height: 1;
  }

  .toggle:hover {
    color: var(--text-primary);
  }

  .toggle:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
