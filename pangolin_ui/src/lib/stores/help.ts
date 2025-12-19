import { writable } from 'svelte/store';

interface HelpState {
    isOpen: boolean;
    docPath: string | null;
    title: string | null;
}

const initialState: HelpState = {
    isOpen: false,
    docPath: null,
    title: null
};

function createHelpStore() {
    const { subscribe, set, update } = writable<HelpState>(initialState);

    return {
        subscribe,
        open: (path: string, title?: string) => {
            update(state => ({
                ...state,
                isOpen: true,
                docPath: path,
                title: title || 'Documentation'
            }));
        },
        close: () => {
            update(state => ({
                ...state,
                isOpen: false
            }));
        },
        toggle: () => {
            update(state => ({
                ...state,
                isOpen: !state.isOpen
            }));
        }
    };
}

export const helpStore = createHelpStore();
