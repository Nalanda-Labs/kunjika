<script lang="ts">
    import { onMount } from 'svelte';

    const phrases = ['No more Slack archaeology.', 'Signal over noise.', 'Built for builders.'];
    let typed = $state('');

    onMount(() => {
        let pi = 0;
        let ci = 0;
        let deleting = false;
        let timer: ReturnType<typeof setTimeout>;

        function tick() {
            const phrase = phrases[pi];
            if (!deleting) {
                typed = phrase.slice(0, ++ci);
                if (ci === phrase.length) {
                    deleting = true;
                    timer = setTimeout(tick, 2200);
                    return;
                }
                timer = setTimeout(tick, 68);
            } else {
                typed = phrase.slice(0, --ci);
                if (ci === 0) {
                    deleting = false;
                    pi = (pi + 1) % phrases.length;
                    timer = setTimeout(tick, 400);
                    return;
                }
                timer = setTimeout(tick, 32);
            }
        }
        timer = setTimeout(tick, 1200);
        return () => clearTimeout(timer);
    });

    const stack = [
        {
            name: 'Rust',
            role: 'Memory-safe, zero-cost abstractions. The backend runs fast because it has to.',
            badge: 'systems lang',
            color: '#ce422b'
        },
        {
            name: 'ntex',
            role: "Async I/O framework built on Rust's actor model. Handles thousands of concurrent connections.",
            badge: 'web framework',
            color: '#6b4fbb'
        },
        {
            name: 'Svelte',
            role: 'Compiled, no virtual DOM. The UI ships as lean vanilla JS — fast on day one, fast forever.',
            badge: 'frontend',
            color: '#ff3e00'
        },
        {
            name: 'YugabyteDB',
            role: 'Distributed SQL that speaks Postgres. Horizontally scalable, resilient, ACID-compliant.',
            badge: 'database',
            color: '#1a6fcb'
        }
    ];

    const features = [
        {
            title: ['permanent answers,', 'not buried threads'],
            desc: 'Every resolved question becomes a reference. Searchable, linkable, ranked by community votes — not buried under 200 Slack messages from 2021.'
        },
        {
            title: ['tags that actually', 'organize things'],
            desc: 'Lightweight tag taxonomy — no nested folders, no admin overhead. Topics emerge from usage. Filter by project, stack, or team instantly.'
        },
        {
            title: ["vote what's right,", "not what's newest"],
            desc: 'Upvotes surface the best answer to the top. No more scrolling past three "have you tried restarting?" replies to find the one that works.'
        },
        {
            title: ['self-host or', 'deploy with docs'],
            desc: 'Runs on a $5 VPS or your existing infra. No vendor lock-in, no SaaS pricing ratchet. Your data, your rules.'
        }
    ];

    const stats = [
        { n: '94%', l: 'questions answered' },
        { n: '3×', l: 'faster than wiki search' },
        { n: '0', l: 'config required to start' }
    ];
</script>

<svelte:head>
    <title>Home ❤ Kunjika</title>
</svelte:head>

<div class="-mx-4 bg-paper font-light leading-relaxed text-ink">
    <!-- HERO -->
    <section class="relative mx-auto flex min-h-[70vh] max-w-[1100px] flex-col justify-center px-5 pt-16 pb-12 md:min-h-screen md:px-14">
        <p class="mb-10 font-mono text-2xl tracking-[0.14em] text-amber/80 md:text-[2rem]">
            minimal qa &amp; forum software
        </p>
        <h1 class="max-w-[800px] font-mono text-[clamp(2.2rem,5vw,3.8rem)] leading-[1.15] font-normal tracking-tight">
            Ask anything.<br />
            Find it <span class="text-amber">forever</span>.<br />
            <span class="block min-h-[1.15em]"
                >{typed}<span class="ml-0.5 inline-block h-[0.85em] w-[3px] animate-blink bg-amber align-text-bottom"
                ></span></span
            >
        </h1>
        <p class="mt-6 mb-12 max-w-[480px] leading-[1.8] text-ink-muted">
            Kunjika keeps your team's knowledge where it belongs — in a searchable, votable, permanent
            thread. No more Slack archaeology.
        </p>
        <div class="flex flex-wrap items-center gap-4">
            <a
                href="https://github.com/Nalanda-Labs/kunjika"
                target="_blank"
                rel="noopener noreferrer"
                class="border border-amber bg-amber px-5 py-2 font-mono text-xs font-medium tracking-[0.06em] text-paper hover:bg-[#c48430]"
                >Start your forum</a
            >
            <a
                href="/questions"
                class="border border-hint px-5 py-2 font-mono text-xs tracking-[0.06em] text-ink-muted hover:border-ink-muted hover:text-ink"
                >View demo →</a
            >
        </div>
    </section>

    <!-- TERMINAL DEMO -->
    <div class="mx-5 my-8 max-w-[780px] border border-hint bg-paper-surface md:mx-14 md:my-16">
        <div class="flex items-center gap-1.5 border-b border-hint bg-paper-raised px-4 py-2.5">
            <span class="h-2.5 w-2.5 rounded-full bg-hint"></span>
            <span class="h-2.5 w-2.5 rounded-full bg-hint"></span>
            <span class="h-2.5 w-2.5 rounded-full bg-hint"></span>
            <span class="ml-1.5 font-mono text-[0.68rem] tracking-[0.04em] text-ink-muted"
                >kunjika.yourteam.io — questions</span
            >
        </div>
        <div class="p-6 font-mono text-[0.82rem] leading-[1.9]">
            <div>How do we handle secrets rotation without downtime?</div>
            <div class="pl-6 text-[0.72rem] text-ink-muted">
                <span class="mr-1 border border-amber/20 bg-amber/10 px-1.5 text-[0.68rem] text-amber">ops</span><span
                    class="mr-1 border border-amber/20 bg-amber/10 px-1.5 text-[0.68rem] text-amber">security</span
                > · asked by arjun · 2 days ago
            </div>
            <div class="h-3"></div>
            <div class="pl-6 text-[#1e7a62]">
                <span class="text-amber">▲ 24 </span>Use HashiCorp Vault's dynamic secrets — rotate at the
                source, not the config. Services get short-lived creds they renew automatically.
            </div>
            <div class="pl-6 text-[0.72rem] text-ink-muted">answered by riya · accepted ✓</div>
            <div class="h-3"></div>
            <div>When should I prefer event sourcing over CRUD?</div>
            <div class="pl-6 text-[0.72rem] text-ink-muted">
                <span class="mr-1 border border-amber/20 bg-amber/10 px-1.5 text-[0.68rem] text-amber">architecture</span
                > · asked by priya · 5 hours ago · <span class="text-[#1e7a62]">3 answers</span>
            </div>
            <div class="h-3"></div>
            <div>Is JSONB safe for high-frequency writes on large tables?</div>
            <div class="pl-6 text-[0.72rem] text-ink-muted">
                <span class="mr-1 border border-amber/20 bg-amber/10 px-1.5 text-[0.68rem] text-amber">postgres</span
                > · asked by dev_k · 1 hour ago · awaiting answers
            </div>
        </div>
    </div>

    <!-- TECH STACK -->
    <section class="mx-auto max-w-[1100px] px-5 py-16 md:px-14">
        <p class="mb-10 font-mono text-2xl tracking-[0.14em] text-ink-muted/70 md:text-[2rem]">built with precision</p>
        <div class="grid grid-cols-1 gap-px border border-hint bg-hint sm:grid-cols-2 lg:grid-cols-4">
            {#each stack as s (s.name)}
                <div class="relative flex flex-col gap-4 overflow-hidden bg-paper px-7 py-8 hover:bg-paper-surface">
                    <span class="absolute inset-x-0 top-0 h-0.5" style="background:{s.color}"></span>
                    <div class="font-mono text-base font-medium">{s.name}</div>
                    <div class="font-mono text-[0.72rem] leading-[1.6] tracking-[0.04em] text-ink-muted">{s.role}</div>
                    <span
                        class="mt-auto self-start border px-2 py-0.5 font-mono text-[0.62rem] tracking-[0.06em]"
                        style="color:{s.color};border-color:{s.color}4d;background:{s.color}0f">{s.badge}</span
                    >
                </div>
            {/each}
        </div>
    </section>

    <!-- FEATURES -->
    <section class="mx-auto max-w-[1100px] px-5 py-16 md:px-14 md:py-24">
        <p class="mb-14 font-mono text-2xl tracking-[0.14em] text-amber/70 md:text-[2rem]">what makes it different</p>
        <div class="grid grid-cols-1 gap-px border border-hint bg-hint md:grid-cols-2">
            {#each features as f, i (i)}
                <div class="bg-paper p-10 transition-colors hover:bg-paper-raised">
                    <div class="mb-5 font-mono text-[0.68rem] tracking-[0.06em] text-amber-dim">0{i + 1}</div>
                    <div class="mb-3 font-mono text-[1.05rem] leading-tight font-normal">
                        {f.title[0]}<br />{f.title[1]}
                    </div>
                    <p class="text-[0.88rem] leading-[1.85] text-ink-muted">{f.desc}</p>
                </div>
            {/each}
        </div>
    </section>

    <!-- STATS -->
    <div class="grid grid-cols-1 border-y border-hint md:grid-cols-3">
        {#each stats as s (s.l)}
            <div class="border-b border-hint px-5 py-8 text-center last:border-0 md:border-r md:border-b-0 md:px-14 md:py-12">
                <div class="mb-2 font-mono text-5xl leading-none font-normal text-amber">{s.n}</div>
                <div class="font-mono text-[0.78rem] tracking-[0.06em] text-ink-muted">{s.l}</div>
            </div>
        {/each}
    </div>

    <footer class="flex flex-col items-center justify-between gap-6 px-5 py-6 text-center md:flex-row md:px-14 md:py-8">
        <a href="/" class="font-mono text-sm text-ink-muted">kunjika</a>
        <ul class="flex gap-8 font-mono text-[0.72rem] tracking-[0.04em] text-ink-muted">
            <li><a class="hover:text-amber" href="https://github.com/Nalanda-Labs/kunjika">github</a></li>
            <li><a class="hover:text-amber" href="/privacy">privacy</a></li>
            <li><a class="hover:text-amber" href="mailto:shivshankar.dayal@gmail.com">Contact email</a></li>
        </ul>
    </footer>
</div>
