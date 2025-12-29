import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params }) => {
    throw redirect(302, `/login?tenant_id=${params.tenant_id}`);
};
