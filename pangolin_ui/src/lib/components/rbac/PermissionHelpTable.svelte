<script lang="ts">
</script>

<div class="overflow-x-auto border border-gray-200 dark:border-gray-700 rounded-lg">
  <table class="table w-full text-sm">
    <thead class="bg-gray-50 dark:bg-gray-900">
      <tr>
        <th class="p-2 text-left">Scope</th>
        <th class="p-2 text-left">Description</th>
        <th class="p-2 text-left">Resource ID Format</th>
        <th class="p-2 text-left">Available Actions</th>
      </tr>
    </thead>
    <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
      <tr>
        <td class="p-2 font-bold text-primary-600">System</td>
        <td class="p-2">Entire Pangolin Platform</td>
        <td class="p-2 font-mono text-xs text-gray-500">N/A</td>
        <td class="p-2">Admin (Manage Tenants)</td>
      </tr>
      <tr>
        <td class="p-2 font-bold text-primary-600">Tenant</td>
        <td class="p-2">Specific Tenant Environment</td>
        <td class="p-2 font-mono text-xs text-gray-500">N/A (Current Tenant)</td>
        <td class="p-2">Admin, Manage Access, Create Catalog, Manage Discovery</td>
      </tr>
      <tr>
        <td class="p-2 font-bold text-blue-500">Catalog</td>
        <td class="p-2">Specific Catalog</td>
        <td class="p-2 font-mono text-xs">"catalog_name"</td>
        <td class="p-2">Read, Write, Create, Manage Access, Manage Discovery, Delete</td>
      </tr>
      <tr>
        <td class="p-2 font-bold text-yellow-500">Namespace</td>
        <td class="p-2">Specific Namespace</td>
        <td class="p-2 font-mono text-xs">"catalog.namespace"</td>
        <td class="p-2">Read, Write, Create (Tables/Views), Delete</td>
      </tr>
      <tr>
        <td class="p-2 font-bold text-purple-500">Table / View</td>
        <td class="p-2">Specific Table or View</td>
        <td class="p-2 font-mono text-xs">"catalog.namespace.table"</td>
        <td class="p-2">Read (Select), Write (Insert/Update), Delete</td>
      </tr>
    </tbody>
  </table>
</div>
