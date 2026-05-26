<script lang="ts">
  import { Link } from "svelte-routing";
</script>

<div class="create">
  <h1>Crea tu USB Stash</h1>
  <p class="subtitle">Elegí una ubicación y una contraseña maestra para tu nuevo stash.</p>

  <form on:submit|preventDefault>
    <div class="field">
      <label for="path">Ubicación</label>
      <input id="path" type="text" placeholder="/ruta/para/nuevo/stash" />
    </div>

    <div class="field">
      <label for="password">Contraseña maestra</label>
      <input id="password" type="password" placeholder="Mínimo 8 caracteres" />
    </div>

    <div class="field">
      <label for="confirm">Confirmar contraseña</label>
      <input id="confirm" type="password" placeholder="Repetí la contraseña" />
    </div>

    <button type="submit" class="btn btn-primary">Crear stash</button>
  </form>

  <p class="back-link">
    <Link to="/">← Volver</Link>
  </p>
</div>

<style>
  .create {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    max-width: 400px;
    margin: 2rem auto;
  }

  h1 {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .subtitle {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  label {
    font-size: 0.8125rem;
    color: var(--text-secondary);
    font-weight: 500;
  }

  input {
    width: 100%;
  }

  .btn {
    padding: 0.625rem 1rem;
    border: none;
    border-radius: 4px;
    font-size: 0.875rem;
    font-weight: 500;
  }

  .btn-primary {
    background: var(--accent);
    color: #fff;
  }

  .btn-primary:hover {
    opacity: 0.9;
  }

  .back-link {
    font-size: 0.8125rem;
    color: var(--text-secondary);
    text-align: center;
  }
</style>
