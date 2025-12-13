## Visuals

- mobile responsive
- dark mode
- light mode
- system theme
- modern ui
- simple navigation
- use material design theme
- generate a logo of a chibi anime pangolin on an Iceberg to use as a logo throughout the UI, this image and the color scheme can be configured by the root user is they want to white label the UI

## If in no_auth mode

There should only be one user allowed, the root user with only one tenant. So they don't need to log in to see dashboard and have access to everything. This is for development and testing purposes only.

## If Auth Turned On

- All users must login to see anything

### Root User

- credentials defined in env variables
- can create users
- can create tenants
- can create catalogs
- can create namespaces
- can create assets
- can create branches
- can create tags
- can create commits
- can create audit logs
- can delete users
- can delete tenants
- can delete catalogs
- can delete namespaces
- can delete assets
- can delete branches
- can delete tags
- can delete commits
- can delete audit log
- can create/delete warehouses

### Tenant Admins

Can only manage within their own tenant

- can create users
- can create catalogs
- can create namespaces
- can create assets
- can create branches
- can create tags
- can create commits
- can create audit logs
- can delete users
- can delete catalogs
- can delete namespaces
- can delete assets
- can delete branches
- can delete tags
- can delete commits
- can delete audit log
- can create delete warehouses
- have full permissions over all assets

### Tenant User

- Have no privileges
- Can be granted roles or individual privileges
- catalog level privileges (apply to everything in catalog)
    - read
    - write
    - delete
    - create
    - update
    - list
    - all
    - none
    - ingest branching (can create ingest branches that can be merged back into main)
    - experimental branching (can create experimental branches that can't be merged back into main)
- namespace level privileges (apply to everything in namespace)
    - read
    - write
    - delete
    - create
    - update
    - list
    - all
    - none
    - ingest branching (can create ingest branches that can be merged back into main)
    - experimental branching (can create experimental branches that can't be merged back into main)
- individual asset privileges
    - read
    - write
    - delete
    - create
    - update
    - list
    - all
    - none
    - ingest branching (can create ingest branches that can be merged back into main)
    - experimental branching (can create experimental branches that can't be merged back into main)
- tag based privileges (privileges for assets with a certain tag/label/attribute)
    - read
    - write
    - delete
    - create
    - update
    - list
    - all
    - none
    - ingest branching (can create ingest branches that can be merged back into main)
    - experimental branching (can create experimental branches that can't be merged back into main)

- root users and tenant admins can create roles, grant roles privileges and assign roles to users

- root users and tenant admins can tag catalogs, assets, and namespaces

### UI for Apache Iceberg Tables
- should be able to browse snapshot history
- should be able to explore metadata of the table
- should be able to see recent transaction in the table, when they happened, what user made the change, what type of change it

### UI for branching
- see list of branches and what assets the branch tracks
- can merge a branch into another branch
- can merge a branch into main
- can create a new branch from another branch
- can delete a branch
- get notified of a merge conflict if one occurs

### UI for tags
- see list of tags and what assets at what snapshot the tag is applied to
- can create a new tag
- can delete a tag
- can edit a tag

## Business Catalog Features

- An entity type called "BusinessMetadata" should be added and any user should be able to add metadata to assets that include a description property, tags and an arbitary number of key value pairs.

- assets should have a property to determine if they are "discoverable", if discoverable you can do a search and discoverable assets you don't have access to will show up with description and you can click "request access" and it will go to the tenant admin to approve. the tenant admin will have an area in the UI where they can see pending requests and approve or reject them.

- Non-discoverable assets don't show up in search results unless you have permission to see them (any privilege on the asset).

Some of this may require changes to the server, feel free to make those changes to implement these features.