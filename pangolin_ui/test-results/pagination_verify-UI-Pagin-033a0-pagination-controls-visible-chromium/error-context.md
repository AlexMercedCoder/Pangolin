# Page snapshot

```yaml
- generic [ref=e3]:
  - generic [ref=e4]: "[plugin:vite-plugin-svelte:compile] /home/alexmerced/development/personal/Personal/2026/pangolin/pangolin_ui/src/routes/warehouses/+page.svelte:50:3 Unexpected token https://svelte.dev/e/js_parse_error"
  - generic [ref=e5]: +page.svelte:50:3
  - generic [ref=e6]: "48 | goto(`/warehouses/${encodeURIComponent(warehouse.name)}`); 49 | } 50 | || warehouse.storage_config?.['azure.container'] ^ 51 | || warehouse.storage_config?.['gcs.bucket'] 52 | || '-';"
  - generic [ref=e7]:
    - text: Click outside, press Esc key, or fix the code to dismiss.
    - text: You can also disable this overlay by setting
    - code [ref=e8]: server.hmr.overlay
    - text: to
    - code [ref=e9]: "false"
    - text: in
    - code [ref=e10]: vite.config.ts
    - text: .
```