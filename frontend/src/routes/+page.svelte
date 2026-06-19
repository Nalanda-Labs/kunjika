<svelte:head>
	<title>Home ❤ Kunjika</title>
</svelte:head>

<main>
	<!-- HERO -->
<section class="hero">
  <p class="hero-tag">minimal qa &amp; forum software</p>
  <h1 class="hero-title">
    Ask anything.<br>
    Find it<span class="amber"> forever</span>.<br>
    <span class="typed-line" id="typed-line"><span class="typed-cursor"></span></span>
  </h1>
  <p class="hero-sub">
    Kunjika keeps your team's knowledge where it belongs — in a searchable, votable, permanent thread. No more Slack archaeology.
  </p>
  <div class="hero-actions">
    <a href="https://github.com/Nalanda-Labs/kunjika" class="btn-amber" taget="_blank">Start your forum</a>
    <a href="/questions" class="btn-ghost">View demo →</a>
  </div>
</section>

<!-- TERMINAL DEMO -->
<div class="terminal-wrap">
  <div class="terminal-bar">
    <div class="dot"></div>
    <div class="dot"></div>
    <div class="dot"></div>
    <span class="terminal-title">kunjika.yourteam.io — questions</span>
  </div>
  <div class="terminal-body">
    <div><span class="t-prompt"><span class="t-question">How do we handle secrets rotation without downtime?</span></span></div>
    <div class="t-meta"><span class="t-tag">ops</span><span class="t-tag">security</span> · asked by arjun · 2 days ago</div>
    <div class="t-blank"></div>
    <div class="t-answer"><span class="t-vote">▲ 24 </span>Use HashiCorp Vault's dynamic secrets — rotate at the source, not the config. Services get short-lived creds they renew automatically.</div>
    <div class="t-meta t-answer">answered by riya · accepted ✓</div>
    <div class="t-blank"></div>
    <div><span class="t-prompt"><span class="t-question">When should I prefer event sourcing over CRUD?</span></span></div>
    <div class="t-meta"><span class="t-tag">architecture</span> · asked by priya · 5 hours ago · <span style="color: #1E7A62">3 answers</span></div>
    <div class="t-blank"></div>
    <div><span class="t-prompt"><span class="t-question">Is JSONB safe for high-frequency writes on large tables?</span></span></div>
    <div class="t-meta"><span class="t-tag">postgres</span> · asked by dev_k · 1 hour ago · <span style="color:var(--muted)">awaiting answers</span></div>
  </div>
</div>

<!-- SOCIAL PROOF MARQUEE -->
<!--div class="proof-bar">
  <div class="marquee-track" id="marquee">
    <span class="marquee-item"><span class="hi">94%</span> questions answered within 24h</span>
    <span class="marquee-item">average response time <span class="hi">47 min</span></span>
    <span class="marquee-item">self-hostable &amp; open source</span>
    <span class="marquee-item">markdown &amp; code blocks supported</span>
    <span class="marquee-item">vote-based answer ranking</span>
    <span class="marquee-item"><span class="hi">94%</span> questions answered within 24h</span>
    <span class="marquee-item">average response time <span class="hi">47 min</span></span>
    <span class="marquee-item">self-hostable &amp; open source</span>
    <span class="marquee-item">markdown &amp; code blocks supported</span>
    <span class="marquee-item">vote-based answer ranking</span>
  </div>
</div-->


<!-- TECH STACK -->
<section class="stack-section">
  <p class="stack-label">built with precision</p>
  <div class="stack-grid">
    <div class="stack-item stack-rust">
      <div class="stack-name">Rust</div>
      <div class="stack-role">Memory-safe, zero-cost abstractions. The backend runs fast because it has to.</div>
      <span class="stack-badge">systems lang</span>
    </div>
    <div class="stack-item stack-ntex">
      <div class="stack-name">ntex</div>
      <div class="stack-role">Async I/O framework built on Rust's actor model. Handles thousands of concurrent connections.</div>
      <span class="stack-badge">web framework</span>
    </div>
    <div class="stack-item stack-svelte">
      <div class="stack-name">Svelte</div>
      <div class="stack-role">Compiled, no virtual DOM. The UI ships as lean vanilla JS — fast on day one, fast forever.</div>
      <span class="stack-badge">frontend</span>
    </div>
    <div class="stack-item stack-yuga">
      <div class="stack-name">YugabyteDB</div>
      <div class="stack-role">Distributed SQL that speaks Postgres. Horizontally scalable, resilient, ACID-compliant.</div>
      <span class="stack-badge">database</span>
    </div>
  </div>
</section>

<!-- FEATURES -->
<section class="features">
  <p class="section-head">what makes it different</p>
  <div class="feat-grid">
    <div class="feat">
      <div class="feat-index">01</div>
      <div class="feat-title">permanent answers,<br>not buried threads</div>
      <p class="feat-desc">Every resolved question becomes a reference. Searchable, linkable, ranked by community votes — not buried under 200 Slack messages from 2021.</p>
    </div>
    <div class="feat">
      <div class="feat-index">02</div>
      <div class="feat-title">tags that actually<br>organize things</div>
      <p class="feat-desc">Lightweight tag taxonomy — no nested folders, no admin overhead. Topics emerge from usage. Filter by project, stack, or team instantly.</p>
    </div>
    <div class="feat">
      <div class="feat-index">03</div>
      <div class="feat-title">vote what's right,<br>not what's newest</div>
      <p class="feat-desc">Upvotes surface the best answer to the top. No more scrolling past three "have you tried restarting?" replies to find the one that works.</p>
    </div>
    <div class="feat">
      <div class="feat-index">04</div>
      <div class="feat-title">self-host or<br>deploy with docs</div>
      <p class="feat-desc">Runs on a $5 VPS or your existing infra. No vendor lock-in, no SaaS pricing ratchet. Your data, your rules.</p>
    </div>
  </div>
</section>

<!-- STATS -->
<div class="stats-row">
  <div class="stat">
    <div class="stat-n">94%</div>
    <div class="stat-l">questions answered</div>
  </div>
  <div class="stat">
    <div class="stat-n">3×</div>
    <div class="stat-l">faster than wiki search</div>
  </div>
  <div class="stat">
    <div class="stat-n">0</div>
    <div class="stat-l">config required to start</div>
  </div>
</div>

<footer>
  <a href="/" class="footer-logo">kunjika</a>
  <ul class="footer-links">
    <li><a href="https://github.com/Nalanda-Labs/kunjika">github</a></li>
    <li><a href="/privacy">privacy</a></li>
    <li><a href="mailto:shivshankar.dayal@gmail.com">Contact email</li>
  </ul>
</footer>
</main>

<style>
	main {
		margin-top: 0px;
		max-width: 800px;
		margin-left: auto;
		margin-right: auto;
	}
	section {
		margin-bottom: 1.5em;
	}
	*,
	*::before,
	*::after {
		box-sizing: border-box;
		margin: 0;
		padding: 0;
	}

	:root {
		--black: #f5f2ed;
		--surface: #edeae4;
		--raised: #e4e0d8;
		--amber: #a0640a;
		--amber-dim: #c8922e;
		--cream: #1a1814;
		--muted: #6b6560;
		--hint: #d5d0c8;
		--mono: 'JetBrains Mono', 'Fira Code', monospace;
		--sans: 'Inter', system-ui, sans-serif;
	}

	html {
		scroll-behavior: smooth;
	}

	body {
		background: var(--black);
		color: var(--cream);
		font-family: var(--sans);
		font-weight: 300;
		line-height: 1.7;
		-webkit-font-smoothing: antialiased;
		overflow-x: hidden;
	}

	/* ─── NAV ───
    nav {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 1.4rem 3.5rem;
      border-bottom: 1px solid var(--hint);
      position: sticky;
      top: 0;
      background: rgba(245,242,237,0.92);
      backdrop-filter: blur(8px);
      z-index: 100;
    }

    .logo {
      font-family: var(--mono);
      font-size: 1rem;
      font-weight: 400;
      color: var(--cream);
      text-decoration: none;
      letter-spacing: -0.02em;
    }

    .logo .cursor {
      display: inline-block;
      width: 2px;
      height: 1em;
      background: var(--amber);
      margin-left: 1px;
      vertical-align: text-bottom;
      animation: blink 1s step-end infinite;
    }

    @keyframes blink {
      0%,100%{opacity:1}
      50%{opacity:0}
    }

    .nav-links {
      display: flex;
      align-items: center;
      gap: 2rem;
      list-style: none;
    }

    .nav-links a {
      font-size: 0.78rem;
      font-family: var(--mono);
      font-weight: 300;
      color: var(--muted);
      text-decoration: none;
      letter-spacing: 0.04em;
      transition: color 0.15s;
    }

    .nav-links a:hover { color: var(--amber); }

    .nav-actions { display: flex; gap: 0.75rem; align-items: center; }

    .btn-ghost {
      font-family: var(--mono);
      font-size: 0.75rem;
      font-weight: 400;
      letter-spacing: 0.06em;
      color: var(--muted);
      background: transparent;
      border: 1px solid var(--hint);
      padding: 0.5rem 1.2rem;
      cursor: pointer;
      text-decoration: none;
      transition: border-color 0.15s, color 0.15s;
      display: inline-block;
    }

    .btn-ghost:hover { border-color: var(--muted); color: var(--cream); }

    .btn-amber {
      font-family: var(--mono);
      font-size: 0.75rem;
      font-weight: 500;
      letter-spacing: 0.06em;
      color: var(--black);
      background: var(--amber);
      border: 1px solid var(--amber);
      padding: 0.5rem 1.2rem;
      cursor: pointer;
      text-decoration: none;
      transition: background 0.15s;
      display: inline-block;
    }

    .btn-amber:hover { background: #c48430; border-color: #c48430; }
    */
	/* ─── HERO ─── */
	.hero {
		min-height: 100vh;
		display: flex;
		flex-direction: column;
		justify-content: center;
		padding: 5rem 3.5rem 4rem;
		max-width: 1100px;
		margin: 0 auto;
		position: relative;
	}

	.hero-tag {
		font-family: var(--mono);
		font-size: 2rem;
		font-weight: 300;
		letter-spacing: 0.14em;
		color: var(--amber);
		margin-bottom: 2.5rem;
		opacity: 0.8;
	}

	.hero-tag::before {
		opacity: 0.5;
	}

	.hero-title {
		font-family: var(--mono);
		font-size: clamp(2.2rem, 5vw, 3.8rem);
		font-weight: 400;
		line-height: 1.15;
		letter-spacing: -0.03em;
		color: var(--cream);
		margin-bottom: 0;
		max-width: 800px;
	}

	.hero-title .amber {
		color: var(--amber);
	}

	.typed-line {
		display: block;
		min-height: 1.15em;
		position: relative;
	}

	.typed-cursor {
		display: inline-block;
		width: 3px;
		background: var(--amber);
		height: 0.85em;
		vertical-align: text-bottom;
		margin-left: 2px;
		animation: blink 0.9s step-end infinite;
	}

	.hero-sub {
		font-family: var(--sans);
		font-size: 1rem;
		font-weight: 300;
		color: var(--muted);
		max-width: 480px;
		line-height: 1.8;
		margin-bottom: 3rem;
	}

	.hero-actions {
		display: flex;
		gap: 1rem;
		align-items: center;
		flex-wrap: wrap;
	}

	.hero-note {
		font-family: var(--mono);
		font-size: 0.7rem;
		color: var(--muted);
		margin-top: 3.5rem;
		opacity: 0.6;
	}

	.hero-note::before {
		color: var(--amber);
		opacity: 1;
	}

	/* ─── GRID BACKGROUND ─── */
	.hero::before {
		position: absolute;
		inset: 0;
		background-image:
			linear-gradient(var(--hint) 1px, transparent 1px),
			linear-gradient(90deg, var(--hint) 1px, transparent 1px);
		background-size: 48px 48px;
		opacity: 0.3;
		pointer-events: none;
		z-index: -1;
		mask-image: radial-gradient(ellipse 80% 60% at 60% 50%, white 30%, transparent 80%);
	}

	/* ─── TERMINAL BLOCK ─── */
	.terminal-wrap {
		background: var(--surface);
		border: 1px solid var(--hint);
		margin: 4rem 3.5rem;
		max-width: 780px;
		position: relative;
	}

	.terminal-bar {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 0.6rem 1rem;
		border-bottom: 1px solid var(--hint);
		background: var(--raised);
	}

	.dot {
		width: 10px;
		height: 10px;
		border-radius: 50%;
		background: var(--hint);
	}

	.terminal-title {
		font-family: var(--mono);
		font-size: 0.68rem;
		color: var(--muted);
		margin-left: 6px;
		letter-spacing: 0.04em;
	}

	.terminal-body {
		padding: 1.5rem 1.5rem;
		font-family: var(--mono);
		font-size: 0.82rem;
		line-height: 1.9;
	}

	.t-prompt {
		color: var(--muted);
	}
	.t-prompt::before {
		color: var(--amber);
	}
	.t-question {
		color: var(--cream);
	}
	.t-answer {
		color: #1e7a62;
		padding-left: 1.5rem;
	}
	.t-meta {
		color: var(--muted);
		font-size: 0.72rem;
		padding-left: 1.5rem;
	}
	.t-tag {
		display: inline-block;
		background: rgba(160, 100, 10, 0.08);
		color: var(--amber);
		font-size: 0.68rem;
		padding: 1px 6px;
		border: 1px solid rgba(160, 100, 10, 0.2);
		margin-right: 4px;
	}
	.t-blank {
		height: 0.8rem;
	}
	.t-vote {
		color: var(--amber);
	}

	/* ─── FEATURES ─── */
	.features {
		padding: 6rem 3.5rem;
		max-width: 1100px;
		margin: 0 auto;
	}

	.section-head {
		font-family: var(--mono);
		font-size: 2rem;
		letter-spacing: 0.14em;
		color: var(--amber);
		opacity: 0.7;
		margin-bottom: 3.5rem;
	}

	/* .section-head::before {
		content: '/* '; 
	}
	/* .section-head::after {
		content: ' ';
	} */

	.feat-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 1px;
		background: var(--hint);
		border: 1px solid var(--hint);
	}

	.feat {
		background: var(--black);
		padding: 2.5rem;
		position: relative;
		transition: background 0.2s;
	}

	.feat:hover {
		background: var(--raised);
	}

	.feat-index {
		font-family: var(--mono);
		font-size: 0.68rem;
		color: var(--amber-dim);
		margin-bottom: 1.2rem;
		letter-spacing: 0.06em;
	}

	.feat-title {
		font-family: var(--mono);
		font-size: 1.05rem;
		font-weight: 400;
		color: var(--cream);
		margin-bottom: 0.8rem;
		line-height: 1.3;
	}

	.feat-desc {
		font-size: 0.88rem;
		color: var(--muted);
		line-height: 1.85;
	}

	/* ─── MARQUEE / SOCIAL PROOF ─── */
	.proof-bar {
		border-top: 1px solid var(--hint);
		border-bottom: 1px solid var(--hint);
		padding: 1.2rem 0;
		overflow: hidden;
		position: relative;
	}

	.marquee-track {
		display: flex;
		gap: 4rem;
		white-space: nowrap;
		animation: scroll 28s linear infinite;
		width: max-content;
	}

	@keyframes scroll {
		from {
			transform: translateX(0);
		}
		to {
			transform: translateX(-50%);
		}
	}

	.marquee-item {
		font-family: var(--mono);
		font-size: 0.72rem;
		color: var(--muted);
		letter-spacing: 0.08em;
	}

	.marquee-item .hi {
		color: var(--amber);
	}

	/* ─── STATS ─── */
	.stats-row {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		border-top: 1px solid var(--hint);
		border-bottom: 1px solid var(--hint);
	}

	.stat {
		padding: 3rem 3.5rem;
		border-right: 1px solid var(--hint);
		text-align: center;
	}

	.stat:last-child {
		border-right: none;
	}

	.stat-n {
		font-family: var(--mono);
		font-size: 3rem;
		font-weight: 400;
		color: var(--amber);
		line-height: 1;
		margin-bottom: 0.5rem;
	}

	.stat-l {
		font-size: 0.78rem;
		color: var(--muted);
		font-family: var(--mono);
		letter-spacing: 0.06em;
	}

	/* ─── FOOTER ─── */
	footer {
		border-top: 1px solid var(--hint);
		padding: 2rem 3.5rem;
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.footer-logo {
		font-family: var(--mono);
		font-size: 0.85rem;
		color: var(--muted);
		text-decoration: none;
	}

	.footer-links {
		display: flex;
		gap: 2rem;
		list-style: none;
	}

	.footer-links a {
		font-family: var(--mono);
		font-size: 0.72rem;
		color: var(--muted);
		text-decoration: none;
		letter-spacing: 0.04em;
		transition: color 0.15s;
	}

	.footer-links a:hover {
		color: var(--amber);
	}

	.footer-copy {
		font-family: var(--mono);
		font-size: 0.68rem;
		color: var(--hint);
		letter-spacing: 0.06em;
	}

	@media (max-width: 768px) {
		nav {
			padding: 1rem 1.2rem;
		}
		.nav-links {
			display: none;
		}
		.hero {
			padding: 4rem 1.2rem 3rem;
			min-height: 70vh;
		}
		.terminal-wrap {
			margin: 2rem 1.2rem;
		}
		.features {
			padding: 4rem 1.2rem;
		}
		.feat-grid {
			grid-template-columns: 1fr;
		}
		.stats-row {
			grid-template-columns: 1fr;
		}
		.stat {
			border-right: none;
			border-bottom: 1px solid var(--hint);
			padding: 2rem 1.2rem;
		}
		.quote-section {
			padding: 4rem 1.2rem;
		}
		.cta {
			padding: 5rem 1.2rem;
		}
		footer {
			flex-direction: column;
			gap: 1.5rem;
			text-align: center;
			padding: 1.5rem 1.2rem;
		}
	}

	/* ─── TECH STACK ─── */
	.stack-section {
		padding: 4rem 3.5rem;
		max-width: 1100px;
		margin: 0 auto;
	}

	.stack-label {
		font-family: var(--mono);
		font-size: 2rem;
		letter-spacing: 0.14em;
		color: var(--muted);
		margin-bottom: 2.5rem;
		opacity: 0.7;
	}

	.stack-label::before {
		color: var(--amber);
		opacity: 1;
	}

	.stack-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 1px;
		background: var(--hint);
		border: 1px solid var(--hint);
	}

	.stack-item {
		background: var(--black);
		padding: 2rem 1.8rem;
		display: flex;
		flex-direction: column;
		gap: 1rem;
		transition: background 0.15s;
		position: relative;
		overflow: hidden;
	}

	.stack-item:hover {
		background: var(--surface);
	}

	.stack-item::before {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		height: 2px;
	}

	.stack-rust::before {
		background: #ce422b;
	}
	.stack-ntex::before {
		background: #6b4fbb;
	}
	.stack-svelte::before {
		background: #ff3e00;
	}
	.stack-yuga::before {
		background: #1a6fcb;
	}

	.stack-name {
		font-family: var(--mono);
		font-size: 1rem;
		font-weight: 500;
		color: var(--cream);
		letter-spacing: -0.01em;
	}

	.stack-role {
		font-family: var(--mono);
		font-size: 0.72rem;
		color: var(--muted);
		letter-spacing: 0.04em;
		line-height: 1.6;
	}

	.stack-badge {
		display: inline-block;
		font-family: var(--mono);
		font-size: 0.62rem;
		padding: 2px 7px;
		border: 1px solid;
		letter-spacing: 0.06em;
		align-self: flex-start;
		margin-top: auto;
	}

	.stack-rust .stack-badge {
		color: #ce422b;
		border-color: rgba(206, 66, 43, 0.3);
		background: rgba(206, 66, 43, 0.06);
	}
	.stack-ntex .stack-badge {
		color: #6b4fbb;
		border-color: rgba(107, 79, 187, 0.3);
		background: rgba(107, 79, 187, 0.06);
	}
	.stack-svelte .stack-badge {
		color: #d4350a;
		border-color: rgba(255, 62, 0, 0.3);
		background: rgba(255, 62, 0, 0.06);
	}
	.stack-yuga .stack-badge {
		color: #1a6fcb;
		border-color: rgba(26, 111, 203, 0.3);
		background: rgba(26, 111, 203, 0.06);
	}

	@media (max-width: 768px) {
		.stack-grid {
			grid-template-columns: repeat(2, 1fr);
		}
		.stack-section {
			padding: 3rem 1.2rem;
		}
	}
</style>
