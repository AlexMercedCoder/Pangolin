<script lang="ts">
</script>

<div class="overflow-x-auto border border-gray-200 dark:border-gray-700 rounded-lg">
  <table class="table w-full text-sm">
    <thead class="bg-gray-50 dark:bg-gray-900">
      <tr>
        <th class="p-2 text-left w-1/4">Action</th>
        <th class="p-2 text-left">Description</th>
        <th class="p-2 text-left w-1/3">Applicable Scopes</th>
      </tr>
    </thead>
    <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
      <tr>
        <td class="p-2 font-mono font-bold text-primary-600">READ</td>
        <td class="p-2">Read data from tables, views, or files.</td>
        <td class="p-2 text-xs">Catalog, Namespace, Table, View</td>
      </tr>
      <tr>
        <td class="p-2 font-mono font-bold text-primary-600">WRITE</td>
        <td class="p-2">Insert, update, or delete data within tables.</td>
        <td class="p-2 text-xs">Catalog, Namespace, Table</td>
      </tr>
      <tr>
        <td class="p-2 font-mono font-bold text-blue-500">LIST</td>
        <td class="p-2">View the existence of catalogs, namespaces, and tables in lists/explorer. Required to browse hierarchy.</td>
        <td class="p-2 text-xs">Tenant, Catalog, Namespace</td>
      </tr>
      <tr>
        <td class="p-2 font-mono font-bold text-green-600">CREATE</td>
        <td class="p-2">Create new namespaces, tables, or views.</td>
        <td class="p-2 text-xs">Catalog, Namespace</td>
      </tr>
      <tr>
        <td class="p-2 font-mono font-bold text-red-500">DELETE</td>
        <td class="p-2">Drop tables, views, or namespaces.</td>
        <td class="p-2 text-xs">Catalog, Namespace, Table, View</td>
      </tr>
      <tr>
        <td class="p-2 font-mono font-bold text-purple-600">MANAGE_DISCOVERY</td>
        <td class="p-2">Edit business metadata (tags, description) and toggle "Discoverable" status.</td>
        <td class="p-2 text-xs">Catalog, Namespace, Table</td>
      </tr>
      <tr>
        <td class="p-2 font-mono font-bold text-red-700">ALL</td>
        <td class="p-2">Full access to valid actions within the scope.</td>
        <td class="p-2 text-xs">System, Tenant, Catalog</td>
      </tr>
    </tbody>
  </table>
</div>
