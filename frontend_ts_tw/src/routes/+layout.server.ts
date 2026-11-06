import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = ({ locals }) => {
    const u = locals.user;
    return {
        user: u && {
            id: u.id,
            username: u.username,
            email: u.email,
            image_url: u.image_url,
            created_date: u.created_date,
            location: u.location,
            designation: u.designation,
            git: u.git,
            website: u.website,
            is_superuser: u.is_superuser,
            is_moderator: u.is_moderator
        }
    };
};
