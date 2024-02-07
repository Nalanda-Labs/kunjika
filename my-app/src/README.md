# Frontend for Kunjika

## Quickstart

`npm i -g pnpm && pnpm i && pnpm dev` will get the dev server up and running on http://localhost:5173.

For production build run `pnpm build && node build` which will get server up and running on http://localhost:3000.

Do not forget to whitelist the schme://host:port on backend.

## Details

This is the frontend of Kunjika, a minimal QA framework or forum. Though it is minimal
I won't shy away from implementing a feature which adds value to the users.

Inspiration has been drawn from Stackoverflow and Discourse. Both are great and if you
are thinking of hosting a forum or QA most probably Discourse is what you should use.
Kunjika is my itch and I am just scratching my itch. I hate the app servers which consume
several hundreds of megabytes of memory when launched. So I decided to write one in Rust.

But this is not about backend. This frontend is done in Svelte. I started web development in
2013 with jQuery. Angular and React came by but I could never get the hang of both. Perhaps
my mind is too simple. Then Svelte came and I was able to learn it quickly and be productive
in it. The performance of Svelte is comparable to Rust UI frameworks like Perseus/Yew.
You can look at benchmarks in Perseus repo.

Perhaps someday I will make another frontend for Kunjika using Perseus but for the the
first implementation it is not worth the pain.