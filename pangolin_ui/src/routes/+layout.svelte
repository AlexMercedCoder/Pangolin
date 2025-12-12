<script>
    import '../app.css';
</script>

<nav>
    <div style="display: flex; justify-content: space-between; align-items: center; max-width: 1200px; margin: 0 auto;">
        <a href="/" style="font-size: 1.25rem;">Pangolin</a>
        <div>
            <a href="/tenants">Tenants</a>
            <a href="/warehouses">Warehouses</a>
            <a href="/assets">Assets</a>
        </div>
    </div>
</nav>

<main>
    <slot />
</main>
