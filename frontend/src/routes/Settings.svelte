<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { navigate } from "svelte-routing";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    changePassword,
    getSettings,
    updateSettings,
    getStashMetadata,
    exportStash,
    type Settings as SettingsType,
    type StashMetadata,
  } from "../lib/api";
  import { settings, isDirty } from "../lib/stores";
  import { createAutoLockTimer } from "../lib/components/AutoLockTimer";

  // Password change
  let currentPassword = "";
  let newPassword = "";
  let passwordError = "";
  let passwordSuccess = "";

  // Auto-lock
  let selectedAutoLock = 300;
  let savedAutoLock = 300;
  $: hasUnsavedChanges = selectedAutoLock !== savedAutoLock;

  // Metadata
  let metadata: StashMetadata | null = null;
  let metadataError = "";

  // Export
  let exportError = "";
  let exportSuccess = "";

  // General
  let loading = true;
  let generalError = "";

  onMount(async () => {
    try {
      const s = await getSettings();
      settings.set(s);
      selectedAutoLock = s.auto_lock_seconds;
      savedAutoLock = s.auto_lock_seconds;

      metadata = await getStashMetadata();
    } catch (e) {
      generalError = `Failed to load settings: ${e}`;
    } finally {
      loading = false;
    }
  });

  async function handleChangePassword() {
    passwordError = "";
    passwordSuccess = "";

    if (!currentPassword || !newPassword) {
      passwordError = "Both fields are required.";
      return;
    }
    if (newPassword.length < 8) {
      passwordError = "New password must be at least 8 characters.";
      return;
    }

    try {
      await changePassword(currentPassword, newPassword);
      passwordSuccess = "Password changed successfully.";
      currentPassword = "";
      newPassword = "";
    } catch (e) {
      passwordError = `Failed to change password: ${e}`;
    }
  }

  async function handleSaveAutoLock() {
    try {
      await updateSettings({ auto_lock_seconds: selectedAutoLock });
      savedAutoLock = selectedAutoLock;
      settings.set({ auto_lock_seconds: selectedAutoLock });
    } catch (e) {
      generalError = `Failed to save settings: ${e}`;
    }
  }

  async function handleExport() {
    exportError = "";
    exportSuccess = "";

    try {
      const target = await open({
        directory: true,
        multiple: false,
        title: "Select export destination",
      });
      if (!target) return;

      await exportStash(target as string);
      exportSuccess = `Stash exported to ${target}.`;
    } catch (e) {
      exportError = `Failed to export: ${e}`;
    }
  }

  function formatDate(epoch: number): string {
    return new Date(epoch * 1000).toLocaleDateString(undefined, {
      year: "numeric",
      month: "long",
      day: "numeric",
    });
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  }
</script>

<div class="settings">
  <div class="settings-header">
    <button class="btn btn-back" on:click={() => navigate("/explorer")}>
      &larr; Back
    </button>
    <h1>Settings</h1>
  </div>

  {#if loading}
    <p class="loading">Loading settings...</p>
  {:else}
    {#if generalError}
      <div class="error-banner">{generalError}</div>
    {/if}

    <!-- Password Section -->
    <section class="section">
      <h2>Change Password</h2>
      <form on:submit|preventDefault={handleChangePassword}>
        <div class="field">
          <label for="current-pw">Current Password</label>
          <input
            id="current-pw"
            type="password"
            bind:value={currentPassword}
            placeholder="Enter current password"
          />
        </div>
        <div class="field">
          <label for="new-pw">New Password</label>
          <input
            id="new-pw"
            type="password"
            bind:value={newPassword}
            placeholder="Enter new password (min 8 chars)"
          />
        </div>
        {#if passwordError}
          <p class="error">{passwordError}</p>
        {/if}
        {#if passwordSuccess}
          <p class="success">{passwordSuccess}</p>
        {/if}
        <button type="submit" class="btn btn-primary">Change Password</button>
      </form>
    </section>

    <!-- Auto-Lock Section -->
    <section class="section">
      <h2>Auto-Lock</h2>
      <div class="field">
        <label for="auto-lock">Lock after inactivity</label>
        <select
          id="auto-lock"
          bind:value={selectedAutoLock}
          on:change={() => {}}
        >
          <option value={0}>Off</option>
          <option value={60}>1 minute</option>
          <option value={300}>5 minutes</option>
          <option value={900}>15 minutes</option>
        </select>
      </div>
      {#if hasUnsavedChanges}
        <p class="unsaved">Unsaved changes</p>
        <button class="btn btn-primary" on:click={handleSaveAutoLock}>
          Save
        </button>
      {/if}
    </section>

    <!-- Metadata Section -->
    <section class="section">
      <h2>Stash Information</h2>
      {#if metadata}
        <dl class="meta-list">
          <div class="meta-item">
            <dt>Version</dt>
            <dd>{metadata.version}</dd>
          </div>
          <div class="meta-item">
            <dt>Format</dt>
            <dd>{metadata.format}</dd>
          </div>
          <div class="meta-item">
            <dt>Created</dt>
            <dd>{formatDate(metadata.created_at)}</dd>
          </div>
          <div class="meta-item">
            <dt>Total Entries</dt>
            <dd>{metadata.total_entries}</dd>
          </div>
          <div class="meta-item">
            <dt>Container Size</dt>
            <dd>{formatBytes(metadata.dat_size)}</dd>
          </div>
        </dl>
      {:else}
        <p class="error">Failed to load metadata.</p>
      {/if}
    </section>

    <!-- Export Section -->
    <section class="section">
      <h2>Export Stash</h2>
      <p class="help-text">
        Export an encrypted copy of your stash to another location. The copy can
        be opened independently with the same password.
      </p>
      <button class="btn btn-primary" on:click={handleExport}>
        Export to...
      </button>
      {#if exportError}
        <p class="error">{exportError}</p>
      {/if}
      {#if exportSuccess}
        <p class="success">{exportSuccess}</p>
      {/if}
    </section>
  {/if}
</div>

<style>
  .settings {
    max-width: 600px;
    margin: 0 auto;
    padding: 1rem 0;
  }

  .settings-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .settings-header h1 {
    font-size: 1.25rem;
    font-weight: 600;
    margin: 0;
  }

  .loading {
    color: var(--text-secondary);
    text-align: center;
    padding: 2rem;
  }

  .error-banner {
    background: var(--danger);
    color: white;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    margin-bottom: 1rem;
    font-size: 0.875rem;
  }

  .section {
    margin-bottom: 2rem;
    padding-bottom: 1.5rem;
    border-bottom: 1px solid var(--border);
  }

  .section h2 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0 0 1rem;
    color: var(--text-primary);
  }

  .field {
    margin-bottom: 0.75rem;
  }

  .field label {
    display: block;
    font-size: 0.8125rem;
    color: var(--text-secondary);
    margin-bottom: 0.25rem;
  }

  .field input,
  .field select {
    width: 100%;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-tertiary);
    color: var(--text-primary);
    font-size: 0.875rem;
    font-family: var(--font-sans);
  }

  .field input:focus,
  .field select:focus {
    outline: none;
    border-color: var(--accent);
  }

  .btn {
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-tertiary);
    color: var(--text-primary);
    font-size: 0.8125rem;
    font-family: var(--font-sans);
    cursor: pointer;
  }

  .btn:hover {
    border-color: var(--accent);
  }

  .btn-primary {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }

  .btn-primary:hover {
    opacity: 0.9;
  }

  .btn-back {
    padding: 0.25rem 0.5rem;
    font-size: 0.75rem;
  }

  .error {
    color: var(--danger);
    font-size: 0.8125rem;
    margin: 0.5rem 0;
  }

  .success {
    color: var(--success, #22c55e);
    font-size: 0.8125rem;
    margin: 0.5rem 0;
  }

  .unsaved {
    color: var(--warning, #f59e0b);
    font-size: 0.8125rem;
    font-style: italic;
    margin: 0.5rem 0;
  }

  .meta-list {
    display: grid;
    gap: 0.5rem;
  }

  .meta-item {
    display: flex;
    justify-content: space-between;
    padding: 0.375rem 0;
    border-bottom: 1px solid var(--border);
  }

  .meta-item dt {
    color: var(--text-secondary);
    font-size: 0.8125rem;
  }

  .meta-item dd {
    color: var(--text-primary);
    font-size: 0.8125rem;
    font-family: var(--font-mono);
    margin: 0;
  }

  .help-text {
    font-size: 0.8125rem;
    color: var(--text-secondary);
    margin: 0 0 0.75rem;
    line-height: 1.5;
  }
</style>
