# General Overview

The **Pangolin Management UI** is a professional, high-performance web interface designed to provide total visibility and control over your data lakehouse.

## 🎨 Design & Theme

Pangolin features a modern, clean aesthetic tailored for data engineers and administrators.

- **Dark/Light Mode**: Toggle between themes using the icon in the top header. The UI defaults based on your system preference.
- **Glassmorphism**: Subtle transparency and backdrop blurs are used for modals and sidebars to maintain context.
- **Responsive Design**: The sidebar collapses on smaller screens to maximize the data viewing area.

## 🏗️ Interface Layout

The interface is divided into three main zones:

### 1. Unified Sidebar (Left)
Your primary navigation hub. It dynamically updates based on your **User Role**:
- **Root**: System-wide management (Tenants, System Settings).
- **Admin**: Tenant resources (Users, Catalogs, Warehouses).
- **User**: Data focused (Explorer, Discovery).

### 2. Global Header (Top)
Provides constant access to:
- **Search**: Quick jump to assets or documentation.
- **Notifications**: Toast alerts for operation statuses (e.g., "Table Created Successfully").
- **User Profile**: Access to your tokens, settings, and logout.
- **Theme Toggle**: Switch between light and dark modes.

### 3. Workspace Area (Center)
The main area where you interact with data, forms, and logs.

## 📊 Dashboard Views

### Root Dashboard
*Accessible only to system owners.*
Provides high-level metrics on system health, total tenants, and active users across the entire platform.

### Tenant Dashboard
*The home screen for most users.*
Displays recent activity, quick-access catalogs, and pending access requests (for admins).

## 🔔 Interaction Patterns

- **Toasts**: Non-intrusive popups in the top right confirm successful actions or explain errors.
- **Modals**: Used for focused tasks like creating a new branch or inviting a user.
- **Optimistic UI**: Many actions (like renaming a catalog) update the UI immediately while the backend completes the task, providing a snappy experience.
