# Page snapshot

```yaml
- generic [ref=e9]:
  - generic [ref=e10]: "[plugin:vite-plugin-svelte:compile] /home/alexmerced/development/personal/Personal/2026/pangolin/pangolin_ui/src/routes/warehouses/+page.svelte:50:3 Unexpected token https://svelte.dev/e/js_parse_error"
  - generic [ref=e11]: +page.svelte:50:3
  - generic [ref=e12]: "48 | goto(`/warehouses/${encodeURIComponent(warehouse.name)}`); 49 | } 50 | || warehouse.storage_config?.['azure.container'] ^ 51 | || warehouse.storage_config?.['gcs.bucket'] 52 | || '-';"
  - generic [ref=e13]:
    - text: Click outside, press Esc key, or fix the code to dismiss.
    - text: You can also disable this overlay by setting
    - code [ref=e14]: server.hmr.overlay
    - text: to
    - code [ref=e15]: "false"
    - text: in
    - code [ref=e16]: vite.config.ts
    - text: .
```