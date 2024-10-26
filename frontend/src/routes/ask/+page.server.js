import { redirect } from '@sveltejs/kit';

/** @type {import('./$types').PageServerLoad} */
export async function load({ locals }) {
	if (!locals.user) throw redirect(307, '/questions');
}

/** @type {import('./$types.js').Actions} */
export const actions = {
	ask: async ({ cookies, request }) => {
		let xsrf_token = cookies.get('xsrf_token');
		const data = await request.formData();
		return;
	}
};
