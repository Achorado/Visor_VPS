<script lang="ts">
  import { onMount } from 'svelte';
  import { activeServerId } from '$lib/stores';
  import { api } from '$lib/api';
  import type { Project, ServiceStatus } from '$lib/types';
  import ProjectCard from '$lib/components/ProjectCard.svelte';
  import LogViewer from '$lib/components/LogViewer.svelte';
  import { Plus } from 'lucide-svelte';

  let projects = $state<Project[]>([]);
  let loading = $state(false);
  let logsModal = $state<{ open: boolean; service: string }>({ open: false, service: '' });

  // Add project form
  let showForm = $state(false);
  let form = $state({ name: '', path: '', service_name: '', color: '#22c55e' });
  let saving = $state(false);

  async function loadProjects() {
    if (!$activeServerId) return;
    loading = true;
    try { projects = await api.projects.list($activeServerId); }
    catch (e) { console.error(e); }
    finally { loading = false; }
  }

  async function addProject() {
    if (!$activeServerId || !form.name || !form.path) return;
    saving = true;
    try {
      await api.projects.add({
        server_id: $activeServerId,
        name: form.name,
        path: form.path,
        service_name: form.service_name || undefined,
        color: form.color,
      });
      form = { name: '', path: '', service_name: '', color: '#22c55e' };
      showForm = false;
      await loadProjects();
    } finally {
      saving = false;
    }
  }

  async function removeProject(id: string) {
    await api.projects.remove(id);
    await loadProjects();
  }

  onMount(loadProjects);
  $effect(() => { if ($activeServerId) loadProjects(); });
</script>

<div class="view-shell">
  <div class="view-header">
    <h1>Projects</h1>
    {#if $activeServerId}
      <button class="btn btn-primary btn-sm" onclick={() => showForm = !showForm}>
        <Plus size={14} />
        Add Project
      </button>
    {/if}
  </div>

  {#if !$activeServerId}
    <div class="empty-view">Select a server to manage projects.</div>
  {:else}
    {#if showForm}
      <div class="card add-form fade-in">
        <h3 style="margin-bottom:0.75rem">New Project</h3>
        <div class="form-row">
          <div class="field" style="flex:1">
            <label class="label">Name</label>
            <input class="input" type="text" placeholder="My App" bind:value={form.name} />
          </div>
          <div class="field" style="flex:2">
            <label class="label">Path</label>
            <input class="input" type="text" placeholder="/var/www/myapp" bind:value={form.path} />
          </div>
        </div>
        <div class="form-row">
          <div class="field" style="flex:2">
            <label class="label">systemd Service (optional)</label>
            <input class="input" type="text" placeholder="myapp" bind:value={form.service_name} />
          </div>
          <div class="field">
            <label class="label">Color</label>
            <input class="input" type="color" bind:value={form.color} style="height:38px;cursor:pointer;padding:2px" />
          </div>
        </div>
        <div style="display:flex;gap:0.5rem;justify-content:flex-end;margin-top:0.5rem">
          <button class="btn btn-ghost btn-sm" onclick={() => showForm = false}>Cancel</button>
          <button class="btn btn-primary btn-sm" onclick={addProject} disabled={saving}>
            {saving ? 'Saving…' : 'Save Project'}
          </button>
        </div>
      </div>
    {/if}

    {#if loading && projects.length === 0}
      <div class="projects-grid">
        {#each {length: 3} as _}
          <div class="skeleton" style="height:100px;border-radius:10px"></div>
        {/each}
      </div>
    {:else if projects.length === 0}
      <div class="empty-view">No projects yet. Click "Add Project" to start.</div>
    {:else}
      <div class="projects-grid">
        {#each projects as project (project.id)}
          <ProjectCard
            {project}
            serverId={$activeServerId}
            onRemove={removeProject}
            onShowLogs={(svc) => logsModal = { open: true, service: svc }}
          />
        {/each}
      </div>
    {/if}
  {/if}
</div>

{#if logsModal.open && $activeServerId}
  <div class="modal-overlay" onclick={() => logsModal.open = false}>
    <div class="modal" style="max-width:800px;height:70vh;display:flex;flex-direction:column"
      onclick={(e) => e.stopPropagation()}>
      <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:1rem">
        <h2>Logs — {logsModal.service}</h2>
        <button class="btn btn-ghost btn-sm" onclick={() => logsModal.open = false}>✕ Close</button>
      </div>
      <div style="flex:1;overflow:hidden">
        <LogViewer serverId={$activeServerId} serviceName={logsModal.service} autoRefresh />
      </div>
    </div>
  </div>
{/if}

<style>
  .view-shell { display: flex; flex-direction: column; height: 100%; padding: 1.5rem; gap: 1rem; overflow-y: auto; }
  .view-header { display: flex; align-items: center; justify-content: space-between; }
  .view-header h1 { font-size: 1.3rem; }
  .projects-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 0.875rem; }
  .add-form { margin-bottom: 0.5rem; }
  .form-row { display: flex; gap: 0.75rem; margin-bottom: 0.5rem; }
  .field { display: flex; flex-direction: column; }
  .empty-view { display: flex; align-items: center; justify-content: center; flex: 1; color: var(--text-muted); font-size: 0.85rem; }
</style>
