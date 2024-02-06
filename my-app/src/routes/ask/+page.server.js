import { redirect } from '@sveltejs/kit';
import * as api from '../../lib/api.js';

/** @type {import('./$types').PageServerLoad} */
export async function load({ locals }) {
	if (!locals.user) throw redirect(307, '/questions');
}

/** @type {import('./$types.js').Actions} */
export const actions = {
	ask: async ({ cookies, request }) => {
		let xsrf_token = cookies.get('xsrf_token'); 
        const data = await request.formData();
        console.log(data);
        console.log(data.get('title'), data.get('body'), data.get('tags'), xsrf_token);
        return;

        // if ($page.data.user && xsrf_token) {
        //     if (question.title < 6 || question.title > 256) {
        //         M.toast({
        //             html: "Title should not be less than 6 or more than 256 characters.",
        //         });
        //         return;
        //     }
        //     if (value.length < 20 || value.length > 100000) {
        //         M.toast({
        //             html: "question should not be less than 20 or more than 100000 characters.",
        //         });
        //         return;
        //     }

        //     question.body = value;

        //     if (question.tagList.length < 1) {
        //         M.toast({ html: "At least one tag should be supplied." });
        //     }

        //     const response = await api.post(
        //         "create-question",
        //         { "title": question.title, "description": question.body, "tag_list": question.tagList },
        //         $session.user.xsrf_token
        //     );

        //     if (response.code === 200 && response.data.id && response.data.slug) {
        //         id = response.data.id;
        //         await goto(`/questions/${id}`);
        //     } else if(response.code === 400) {
        //         M.toast({html: response.msg});
        //     }
        // } else {
        //     M.toast({ html: "You are not logged in." });
        // }
		// throw redirect(307, '/questions');
	}
}