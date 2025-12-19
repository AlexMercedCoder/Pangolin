import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import fs from 'fs/promises';
import path from 'path';

export const GET: RequestHandler = async ({ params }) => {
    const docPath = params.path;
    
    if (!docPath) {
        throw error(400, 'Path is required');
    }

    // Security check: Ensure we stay within the docs directory
    if (docPath.includes('..') || path.isAbsolute(docPath)) {
        throw error(403, 'Invalid path');
    }

    // Resolve the absolute path to the docs directory
    // SvelteKit app is in pangolin_ui, docs is in the parent directory
    const docsDir = path.resolve(process.cwd(), '../docs');
    const fullPath = path.join(docsDir, docPath + (docPath.endsWith('.md') ? '' : '.md'));

    try {
        const content = await fs.readFile(fullPath, 'utf-8');
        return json({ content });
    } catch (e: any) {
        if (e.code === 'ENOENT') {
            throw error(404, `Document not found: ${docPath}`);
        }
        throw error(500, `Error reading document: ${e.message}`);
    }
};
