<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { theme, loadServers, activeTab } from '$lib/stores';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import AddServerModal from '$lib/components/AddServerModal.svelte';
  import Dashboard from '$lib/views/Dashboard.svelte';
  import ServicesView from '$lib/views/ServicesView.svelte';
  import ProjectsView from '$lib/views/ProjectsView.svelte';
  import ProcessesView from '$lib/views/ProcessesView.svelte';
  import ContainersView from '$lib/views/ContainersView.svelte';
  import NginxView from '$lib/views/NginxView.svelte';
  import SettingsView from '$lib/views/SettingsView.svelte';

  let showAddServer = $state(false);

  onMount(() => {
    theme.init();
    loadServers();
  });
</script>

<div class="app-shell">
  <Sidebar onAddServer={() => showAddServer = true} />

  <main class="main-content">
    {#if $activeTab === 'dashboard'}
      <Dashboard />
    {:else if $activeTab === 'services'}
      <ServicesView />
    {:else if $activeTab === 'projects'}
      <ProjectsView />
    {:else if $activeTab === 'processes'}
      <ProcessesView />
    {:else if $activeTab === 'containers'}
      <ContainersView />
    {:else if $activeTab === 'nginx'}
      <NginxView />
    {:else if $activeTab === 'settings'}
      <SettingsView />
    {/if}
  </main>
</div>

{#if showAddServer}
  <AddServerModal onClose={() => showAddServer = false} />
{/if}

<style>
  .app-shell {
    display: flex;
    height: 100vh;
    overflow: hidden;
    background: var(--bg-base);
  }
  .main-content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>
