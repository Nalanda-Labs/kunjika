<script lang="ts">
    import { onMount } from 'svelte';
    import * as api from '#lib/api.ts';

    let { id }: { id: string } = $props();

    let karma = $state(0);
    let questions = $state(0);
    let answers = $state(0);

    onMount(async () => {
        const res = await api.get(`${id}/summary`);
        if (res.status !== 200) return;
        const j = await api.json<{ data: { karma: number; questions_count: number | null; answers_count: number | null } }>(res);
        karma = j.data.karma;
        questions = j.data.questions_count ?? 0;
        answers = j.data.answers_count ?? 0;
    });

    let cards = $derived([
        { title: 'Karma', text: `${karma} karma earned.` },
        { title: 'Questions', text: `${questions} questions asked.` },
        { title: 'Answers', text: `${answers} answers.` },
        { title: 'Flags', text: '0 helpful flags.' },
        { title: 'Votes', text: '0 votes cast.' }
    ]);
</script>

<div class="mt-4 grid grid-cols-1 gap-3 sm:grid-cols-2">
    {#each cards as c (c.title)}
        <div class="card">
            <h5 class="text-lg font-normal">{c.title}</h5>
            <p class="text-sm text-gray-700">{c.text}</p>
        </div>
    {/each}
</div>
