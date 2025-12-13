<script lang="ts">
    import { user } from '$lib/stores';
</script>

<div class="dashboard">
    <h1>Dashboard</h1>
    
    <div class="welcome-section">
        <h2>Welcome, {$user?.username || 'Guest'}!</h2>
        <p>This is your Pangolin Lakehouse Catalog.</p>
    </div>

    <div class="stats-grid">
        <div class="stat-card">
            <span class="material-icons">inventory_2</span>
            <div class="stat-info">
                <span class="value">0</span>
                <span class="label">Total Assets</span>
            </div>
        </div>
        <div class="stat-card">
            <span class="material-icons">group</span>
            <div class="stat-info">
                <span class="value">0</span>
                <span class="label">Users</span>
            </div>
        </div>
         <div class="stat-card">
            <span class="material-icons">lock</span>
            <div class="stat-info">
                <span class="value">0</span>
                <span class="label">Access Requests</span>
            </div>
        </div>
    </div>
</div>

<style>
    .dashboard {
        padding: 1rem;
    }
    
    h1 {
        font-size: 2rem;
        margin-bottom: 2rem;
        color: var(--md-sys-color-primary);
    }

    .welcome-section {
        margin-bottom: 3rem;
        background-color: var(--md-sys-color-surface-container);
        padding: 2rem;
        border-radius: 16px;
    }

    .stats-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
        gap: 1.5rem;
    }

    .stat-card {
        background-color: var(--md-sys-color-surface);
        padding: 1.5rem;
        border-radius: 16px;
        display: flex;
        align-items: center;
        gap: 1rem;
        box-shadow: 0 2px 4px rgba(0,0,0,0.05);
        border: 1px solid var(--md-sys-color-outline-variant);
    }

    .stat-card .material-icons {
        font-size: 3rem;
        color: var(--md-sys-color-primary);
        opacity: 0.8;
    }

    .stat-info {
        display: flex;
        flex-direction: column;
    }

    .value {
        font-size: 2rem;
        font-weight: bold;
        color: var(--md-sys-color-on-surface);
    }

    .label {
        font-size: 0.875rem;
        color: var(--md-sys-color-outline);
    }
</style>
