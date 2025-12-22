import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import RoleAssignment from './RoleAssignment.svelte';

describe('RoleAssignment', () => {
  const roles = [
    { id: '1', name: 'Admin', description: 'Admin role' },
    { id: '2', name: 'User', description: 'User role' },
    { id: '3', name: 'Viewer', description: 'Viewer role' }
  ];

  it('renders available and assigned roles correctly', () => {
    const { getByText, queryByText } = render(RoleAssignment, { 
      availableRoles: roles, 
      assignedRoleIds: ['1'] 
    });

    // Assigned
    expect(getByText('Admin')).toBeTruthy();
    // Available
    expect(getByText('User')).toBeTruthy();
    expect(getByText('Viewer')).toBeTruthy();
  });

  it('moves role from available to assigned on click', async () => {
    const { getByText, getAllByText, queryByText } = render(RoleAssignment, { 
      availableRoles: roles, 
      assignedRoleIds: [] 
    });
    
    // Initial: 3 available roles (3 "+ Assign" badges)
    expect(getAllByText('+ Assign').length).toBe(3);
    
    // Click 'Admin' (Available)
    // Note: getByText('Admin') might find generic text inside button.
    await fireEvent.click(getByText('Admin'));

    // After click: Admin moves to assigned.
    // Should contain "REMOVE" badge (1 assigned role)
    expect(getByText('REMOVE')).toBeTruthy();
    // Should have 2 available roles left
    expect(getAllByText('+ Assign').length).toBe(2);
  });

  it('moves role from assigned to available on click', async () => {
    const { getByText, getAllByText } = render(RoleAssignment, { 
      availableRoles: roles, 
      assignedRoleIds: ['1'] 
    });

    // Initial: 1 assigned (Admin), 2 available (User, Viewer)
    expect(getByText('REMOVE')).toBeTruthy();
    expect(getAllByText('+ Assign').length).toBe(2);

    // Click 'Admin' in assigned list
    await fireEvent.click(getByText('Admin'));

    // After click: Admin moves back to available.
    // "REMOVE" badge should be gone (or checking "No roles assigned")
    // Note: Svelte might keep it in DOM if checking specific text, 
    // but here we check for the Badge existence specifically.
    // If assigned list is empty, "REMOVE" badge should not exist.
    expect(document.body.textContent).not.toContain('REMOVE'); 

    // Should have 3 available roles now
    expect(getAllByText('+ Assign').length).toBe(3);
  });
});
