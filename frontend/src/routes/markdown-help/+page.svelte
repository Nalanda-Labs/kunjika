<svelte:head>
    <title>Markdown Help ❤ Kunjika</title>
    <!-- Load MathJax v4 so the example formulas render live on this page.
         Config must precede the component script. Single-`$` inline delimiters
         are enabled explicitly (v4 doesn't turn them on by default).
         MathJax skips <pre>/<code> by default, so the copy-paste source blocks
         below stay literal while the rendered preview divs get typeset. -->
    <script>
        window.MathJax = {
            tex: {
                inlineMath: [
                    ['$', '$'],
                    ['\\(', '\\)']
                ],
                displayMath: [
                    ['$$', '$$'],
                    ['\\[', '\\]']
                ]
            },
            startup: {
                ready: () => {
                    window.MathJax.startup.defaultReady();
                    window.MathJax.startup.promise.then(() => window.MathJax.typesetPromise());
                }
            }
        };
    </script>
    <script defer src="https://cdn.jsdelivr.net/npm/mathjax@4/tex-chtml.js"></script>
</svelte:head>
<script>
    // Reference examples for the MathJax (LaTeX) formulas supported on
    // question pages. Rendered as plain text so users can copy the source.
    // `String.raw` keeps backslashes literal (LaTeX commands).
    const mathExamples = [
        {
            title: 'Inline and display math',
            code: String.raw`Inline:   $e^{i\pi} + 1 = 0$
Inline:   \(a^2 + b^2 = c^2\)

Display using $$ ... $$:
$$
\int_{-\infty}^{\infty} e^{-x^{2}}\,dx = \sqrt{\pi}
$$

Display using \[ ... \]:
\[
  \frac{-b \pm \sqrt{b^{2} - 4ac}}{2a}
\]`,
            render: String.raw`$e^{i\pi} + 1 = 0 \qquad \int_{-\infty}^{\infty} e^{-x^{2}}\,dx = \sqrt{\pi} \qquad \dfrac{-b \pm \sqrt{b^{2} - 4ac}}{2a}$`
        },
        {
            title: 'Superscripts, subscripts and primes',
            code: String.raw`$x^{2}$          superscript
$x_i$          subscript
$x^{2}_{i}$    both together
$x^{y^{z}}$    grouped / nested
$x_{i,j}$      multiple subscripts
$\sum_{i=1}^{n}$   limits attach to big operators
$\,{}^{14}C$   prescript (e.g. isotopes)
$f'(x)$, $f''(x)$, $a'$   primes`,
            render: String.raw`$x^{2}$, $x_i$, $x^{2}_{i}$, $x^{y^{z}}$, $x_{i,j}$, $\sum_{i=1}^{n}$, $\,{}^{14}C$, $f'(x)$, $f''(x)$`
        },
        {
            title: 'Greek letters',
            code: String.raw`Lowercase:
\alpha \beta \gamma \delta \epsilon \varepsilon \zeta \eta \theta
\iota \kappa \lambda \mu \nu \xi \omicron \pi \varpi \rho \varrho
\sigma \varsigma \tau \upsilon \phi \varphi \chi \psi \omega

Uppercase:
\Gamma \Delta \Theta \Lambda \Xi \Pi \Sigma \Upsilon \Phi \Psi \Omega`,
            render: String.raw`$\alpha\ \beta\ \gamma\ \delta\ \epsilon\ \varepsilon\ \zeta\ \eta\ \theta\ \iota\ \kappa\ \lambda\ \mu\ \nu\ \xi\ \pi\ \rho\ \sigma\ \tau\ \phi\ \chi\ \psi\ \omega \qquad \Gamma\ \Delta\ \Theta\ \Lambda\ \Xi\ \Pi\ \Sigma\ \Phi\ \Psi\ \Omega$`
        },
        {
            title: 'Fractions and binomials',
            code: String.raw`$\frac{a}{b}$          fraction
$\dfrac{a}{b}$         larger (display) fraction
$\tfrac{a}{b}$         smaller (text) fraction
$a/b$                  slash form
$\binom{n}{k}$         binomial coefficient
$1 + \cfrac{1}{1 + \cfrac{1}{2}}$   continued fraction`,
            render: String.raw`$\frac{a}{b}$, $\dfrac{a}{b}$, $\binom{n}{k}$, $1 + \cfrac{1}{1 + \cfrac{1}{2}}$`
        },
        {
            title: 'Roots',
            code: String.raw`$\sqrt{2}$             square root
$\sqrt[3]{x}$          cube / nth root
$\sqrt{1 + \sqrt{2}}$  nested roots
$\sqrt[n]{\frac{a}{b}}$`,
            render: String.raw`$\sqrt{2}$, $\sqrt[3]{x}$, $\sqrt{1 + \sqrt{2}}$, $\sqrt[n]{\frac{a}{b}}$`
        },
        {
            title: 'Sums, products, limits and integrals',
            code: String.raw`$\sum_{i=1}^{n} i = \frac{n(n+1)}{2}$
$\prod_{i=1}^{n} i = n!$
$\lim_{x \to 0} \frac{\sin x}{x} = 1$
$\lim_{n \to \infty} a_n = L$
$\int_a^b f(x)\,dx$
$\int_0^1 \int_0^1 f(x,y)\,dx\,dy$
$\oint_C \vec{F}\cdot d\vec{r}$      closed line integral
$\iint_D$, $\iiint_V$, $\bigcup_{i=1}^n A_i$, $\bigcap_{i=1}^n A_i$`,
            render: String.raw`$\sum_{i=1}^{n} i = \frac{n(n+1)}{2}$, $\prod_{i=1}^{n} i = n!$, $\lim_{x \to 0} \frac{\sin x}{x} = 1$, $\int_a^b f(x)\,dx$, $\oint_C \vec{F}\cdot d\vec{r}$, $\bigcup_{i=1}^n A_i$`
        },
        {
            title: 'Matrices and arrays',
            code: String.raw`$\begin{matrix} a & b \\ c & d \end{matrix}$

$\begin{pmatrix} 1 & 2 \\ 3 & 4 \end{pmatrix}$   parentheses

$\begin{bmatrix} 1 & 2 \\ 3 & 4 \end{bmatrix}$   square brackets

$\begin{vmatrix} a & b \\ c & d \end{vmatrix}$   determinant

$\begin{Vmatrix} a & b \\ c & d \end{Vmatrix}$   norm

$\begin{array}{cc|c} 1 & 2 & 3 \\ 4 & 5 & 6 \end{array}$   custom columns`,
            render: String.raw`$$\begin{pmatrix} 1 & 2 \\ 3 & 4 \end{pmatrix} \qquad \begin{bmatrix} 1 & 2 \\ 3 & 4 \end{bmatrix} \qquad \begin{vmatrix} a & b \\ c & d \end{vmatrix}$$`
        },
        {
            title: 'Aligning multiple equations',
            code: String.raw`\begin{align}
  (a + b)^2 &= a^2 + 2ab + b^2 \\
  (a - b)^2 &= a^2 - 2ab + b^2
\end{align}`,
            render: String.raw`$$\begin{align} (a+b)^2 &= a^2 + 2ab + b^2 \\ (a-b)^2 &= a^2 - 2ab + b^2 \end{align}$$`
        },
        {
            title: 'Piecewise functions (cases)',
            code: String.raw`f(x) = \begin{cases}
  1   & \text{if } x \geq 0 \\
  -1  & \text{if } x < 0
\end{cases}`,
            render: String.raw`$$f(x) = \begin{cases} 1 & \text{if } x \geq 0 \\ -1 & \text{if } x < 0 \end{cases}$$`
        },
        {
            title: 'Common functions and operators',
            code: String.raw`$\sin x$, $\cos x$, $\tan x$, $\cot x$, $\sec x$, $\csc x$
$\arcsin x$, $\arccos x$, $\arctan x$
$\sinh x$, $\cosh x$, $\tanh x$
$\log x$, $\ln x$, $\log_{2} x$, $\lg x$, $\exp(x)$
$\max$, $\min$, $\sup$, $\inf$, $\arg$, $\deg$, $\det$, $\dim$, $\gcd$, $\ker$, $\Pr$
$\frac{d}{dx}$, $\frac{\partial f}{\partial x}$, $\nabla f$, $\Delta x$`,
            render: String.raw`$\sin x, \cos x, \tan x$, $\arcsin x$, $\sinh x$, $\log x$, $\ln x$, $\exp(x)$, $\frac{d}{dx}$, $\frac{\partial f}{\partial x}$, $\nabla f$, $\Delta x$`
        },
        {
            title: 'Accents and annotations',
            code: String.raw`$\hat{a}$, $\bar{a}$, $\vec{a}$, $\tilde{a}$, $\dot{a}$, $\ddot{a}$
$\widehat{abc}$, $\widetilde{abc}$
$\overline{a + b}$, $\underline{a + b}$
$\overrightarrow{AB}$, $\overleftrightarrow{AB}$
$\overbrace{a + b + c}^{\text{sum}}$, $\underbrace{a + b + c}_{\text{sum}}$`,
            render: String.raw`$\hat{a}$, $\vec{a}$, $\tilde{a}$, $\dot{a}$, $\overline{a + b}$, $\overrightarrow{AB}$, $\overbrace{a + b + c}^{\text{sum}}$`
        },
        {
            title: 'Sets, logic and relations',
            code: String.raw`$\mathbb{R}$, $\mathbb{N}$, $\mathbb{Z}$, $\mathbb{Q}$, $\mathbb{C}$
$\emptyset$, $\varnothing$, $\infty$
$\in$, $\notin$, $\ni$, $\subset$, $\subseteq$, $\supset$, $\supseteq$
$\cup$, $\cap$, $\setminus$, $\times$
$\forall$, $\exists$, $\nexists$, $\neg$, $\land$, $\lor$
$\leq$, $\geq$, $\neq$, $\approx$, $\equiv$, $\sim$, $\cong$, $\propto$, $\ll$, $\gg$
$\pm$, $\mp$, $\cdot$, $\ast$, $\star$, $\circ$, $\bullet$, $\oplus$, $\otimes$`,
            render: String.raw`$\mathbb{R}, \mathbb{N}, \mathbb{Z}$, $\emptyset$, $\infty$, $\in$, $\subset$, $\subseteq$, $\cup$, $\cap$, $\forall$, $\exists$, $\leq$, $\geq$, $\neq$, $\approx$, $\pm$, $\cdot$`
        },
        {
            title: 'Arrows',
            code: String.raw`$\to$, $\gets$, $\leftrightarrow$, $\uparrow$, $\downarrow$
$\Rightarrow$, $\Leftarrow$, $\Leftrightarrow$
$\Longrightarrow$, $\longleftarrow$
$\mapsto$, $\longmapsto$, $\rightsquigarrow$
$x \xrightarrow{f} y$   extensible arrow`,
            render: String.raw`$\to$, $\Rightarrow$, $\Leftrightarrow$, $\mapsto$, $x \xrightarrow{f} y$`
        },
        {
            title: 'Brackets, delimiters and auto-sizing',
            code: String.raw`$(a)$, $[a]$, $\{a\}$, $|a|$, $\|a\|$, $\langle a \rangle$
$\left( \frac{a}{b} \right)$        auto-sized parentheses
$\left[ \frac{a}{b} \right]$        auto-sized brackets
$\left\{ \frac{a}{b} \right\}$      auto-sized braces
$\left\| \frac{a}{b} \right\|$      auto-sized norm
$\left. \frac{df}{dx} \right|_{x=0}$   evaluation bar`,
            render: String.raw`$\left( \frac{a}{b} \right)$, $\left[ \frac{a}{b} \right]$, $\left\| \frac{a}{b} \right\|$, $\left. \frac{df}{dx} \right|_{x=0}$`
        },
        {
            title: 'Fonts and text styles',
            code: String.raw`$\mathbb{R}$   blackboard bold
$\mathbf{v}$   bold
$\mathcal{L}$   calligraphy
$\mathfrak{g}$   fraktur
$\mathrm{d}$    roman (upright)
$\mathit{x}$    italic
$\mathsf{s}$    sans-serif
$\mathtt{c}$    typewriter
$\boldsymbol{\alpha}$   bold symbol
$\textbf{bold}$, $\textit{italic}$, $\texttt{code}$   (inside \text)`,
            render: String.raw`$\mathbb{R}$, $\mathbf{v}$, $\mathcal{L}$, $\mathfrak{g}$, $\mathrm{d}$, $\mathit{x}$, $\mathsf{s}$, $\mathtt{c}$, $\boldsymbol{\alpha}$`
        },
        {
            title: 'Spacing and text inside math',
            code: String.raw`$a\,b$      thin space
$a\;b$      medium space
$a\quad b$  one em
$a\qquad b$  two em
$a\!b$      negative thin space
$\text{if } x > 0$   text inside math
$\text{velocity} = \frac{dx}{dt}$`,
            render: String.raw`$a\,b$, $a\quad b$, $\text{if } x > 0$, $\text{velocity} = \frac{dx}{dt}$`
        },
        {
            title: 'Boxes, cancel and color (extensions)',
            code: String.raw`$\boxed{E = mc^{2}}$

$\require{cancel} \cancel{x} + \bcancel{y} + \xcancel{xyz}$

$\require{color} \color{red}{x^2} + \color{blue}{y^2}$

Note: cancel and color are loaded on demand via \require{...}.`,
            render: String.raw`$$\boxed{E = mc^{2}} \qquad \require{cancel}\cancel{x} + \bcancel{y} \qquad \require{color}\color{red}{x^2} + \color{blue}{y^2}$$`
        }
    ];
</script>
<main style="margin-top:20px">
    <h2>Introduction</h2>
    <p>
        Markdown is a simple format popularized by Stackoverflow. You can read original stuff at
        <a href="https://daringfireball.net/linked/2014/01/08/markdown-extension"
            >Official Markdown Website</a
        >. Most popular variation of markdown is called CommonMark but there is also github
        flavoured markdown. We also use markdown for Kunjika.
    </p>
    <h3>Code and Preformatted Text</h3>
    <p>
        Indent four spaces to create an escaped <code>&lt;pre&gt;</code> and
        <code>&lt;code&gt;</code>
        block:
    </p>
    <pre>
<span style="background-color: #fafafa;margin-left:2px"></span>printf("%d\n", 42);  /* what was the
<span style="background-color: #fafafa;margin-left:2px"
        ></span>                        question again? */</pre>
    <p style="margin-top:5px">
        The text will be wrapped in tags, and displayed in a monospaced font. The first four spaces
        will be stripped off, but all other whitespace will be preserved.
    </p>
    <p>Markdown and HTML are ignored within a code block:</p>
    <pre>
<span style="background-color: #fafafa;margin-left:2px"></span>&lt;blink&gt;
<span style="background-color: #fafafa;margin-left:2px"></span>You would hate this if it weren't
<span style="background-color: #fafafa;margin-left:2px"></span>wrapped in a code block.
<span style="background-color: #fafafa;margin-left:2px"></span>&lt;/blink&gt;</pre>
    <p>
        You can also create code blocks with fenced code consisting of 3 or more tildes followed by
        the language name optionally of the code block
    </p>
    <pre>```python
print("Hello world")
```</pre>
    <h3>Code Spans</h3>
    <p>
        For inline code you can use <code>`code`</code>. An exmaple would be
        <code>`make html`</code>.
    </p>
    <p>
        Like code blocks, code spans will be displayed in a monospaced font. Markdown and HTML will
        not work within them. Note that, unlike code blocks, code spans require you to manually
        escape any HTML within!
    </p>
    <p>
        If your code itself contains backticks, you may have to use multiple backticks as
        delimiters:
    </p>
    <pre>
The name ``Tuple`2`` is a valid .NET type name.</pre>
    <h3>Italics and Bold</h3>
    <pre style="background-color: #ddd;margin-bottom:0">
*This is italicized*, and so is _this_.
**This is bold**, and so is __this__.
Use ***italics and bold together*** if you ___have to___.</pre>
    <h3>Links</h3>
    <pre>
Here's an inline link to [Google](https://www.google.com/).
Here's a reference-style link to [Google][1].
Here's a very readable link to [Yahoo!][yahoo].

  [1]: https://www.google.com/
  [yahoo]: https://www.yahoo.com/</pre>
    <p>
        The link definitions can appear anywhere in the document -- before or after the place where
        you use them. The link definition names <code>[1]</code> and <code>[yahoo]</code> can be any
        unique string, and are case-insensitive; <code>[yahoo]</code> is the same as
        <code>[YAHOO]</code>.
    </p>
    <p>You can also use standard HTML hyperlink syntax.</p>
    <pre>&lt;a href="https://example.com" title="example"&gt;example&lt;/a&gt;</pre>
    <p>
        The editor will automatically convert naked links to clickable URLs(in most but not all
        cases --- beware of unusal characters in your URLs)
    </p>
    <pre>I often visit https://kunjika.ashtavakra.org</pre>
    <h3>Header</h3>
    <p>Use Underline text to make the two <code>&lt;h1&gt;&lt;h2&gt;</code> top-level headers :</p>
    <pre>Header 1
========
Header 2
--------</pre>
    <p>
        The number of = or - signs doesn't matter; one will work. But using enough to underline the
        text makes your titles look better in plain text.
    </p>
    <p>Use hash marks for several levels of headers:</p>
    <pre># Header 1 #
## Header 2 ##
### Header 3 ###</pre>
    <p>The closing # characters are optional.</p>
    <h3>Horizontal Rules</h3>
    <p>
        Insert a horizontal rule <code>&lt;hr/&gt;</code> by putting three or more hyphens, asterisks,
        or underscores on a line by themselves:
    </p>
    <pre>Rule #1
---
Rule #2
*******
Rule #3
___</pre>
    <p>Using spaces between the characters also works:</p>
    <pre>Rule #4
- - - -</pre>
    <h3>Simple Lists</h3>
    <p>A bulleted <code>&lt;ul&gt;</code> list:</p>
    <pre>- Use a minus sign for a bullet
+ Or plus sign
* Or an asterisk</pre>
    <p>A numbered <code>&lt;ol&gt;</code> list:</p>
    <pre>1. Numbered lists are easy
2. Markdown keeps track of the numbers for you
7. So this will be item 3.</pre>
    <p>A double-spaced list:</p>
    <pre>- This list gets wrapped in &lt;p&gt; tags

- So there will be extra space between items</pre>
    <h3>Nesting in lists</h3>
    <p>To put other Markdown blocks in a list, just indent four spaces for each nesting level:</p>
    <pre>1. Lists in a list item:
    - Indented four spaces.
        * indented eight spaces.
    - Four spaces again.
2.  Multiple paragraphs in a list items:
    It's best to indent the paragraphs four spaces
    You can get away with three, but it can get
    confusing when you nest other things.
    Stick to four.

    We indented the first line an extra space to align
    it with these paragraphs. In real use, we might do
    that to the entire list so that all items line up.

    This paragraph is still part of the list item, but it looks messy to humans. So it's a good idea to wrap your nested paragraphs manually, as we did with the first two.

3. Blockquotes in a list item:

    &gt; Skip a line and
    &gt; indent the &gt;'s four spaces.

4. Preformatted text in a list item:

        Skip a line and indent eight spaces.
        That's four spaces for the list
        and four to trigger the code block.</pre>
    <h3>Simple Blockquotes</h3>
    <p>Add a > to the beginning of any line to create a blockquote.</p>
    <pre>> The syntax is based on the way email programs
> usually do quotations. You don't need to hard-wrap
> the paragraphs in your blockquotes, but it looks much nicer if you do.  Depends how lazy you feel.</pre>
    <p>
        To put other Markdown blocks in a blockquote, just add a <code>></code> followed by a space:
    </p>
    <pre>> The `>` on the blank lines is required
> to create a single blockquote.
>
> If you leave out the extra `>`
> you will end up with
> two distinct blockquotes.</pre>
    <p>Blockquotes within a blockquote:</p>
    <pre>> A standard blockquote is indented
> > A nested blockquote is indented more
> > > > You can nest to any depth.</pre>
    <p>Lists in a blockquote:</p>
    <pre>> - A list in a blockquote
> - With a &gt; and space in front of it
>    * A sublist</pre>
    <p>Preformatted text in a blockquote:</p>
    <pre>>     Indent five spaces total.  The first
>     one is part of the blockquote designator.</pre>
    <h3>Images</h3>
    <p>Images are exactly like links, but they have an exclamation point in front of them:</p>
    <pre>![Valid XHTML](https://w3.org/Icons/valid-xhtml10).</pre>
    <p>
        The word in square brackets is the alt text, which gets displayed if the browser can't show
        the image. Be sure to include meaningful alt text for screen-reading software.
    </p>

    <p>Just like links, images work with reference syntax and titles:</p>
    <pre>This page is ![valid XHTML][checkmark].

[checkmark]: https://w3.org/Icons/valid-xhtml10
             "What are you smiling at?"</pre>
    <p>
        You can also use standard HTML image syntax, which allows you to scale the width and height
        of the image.
    </p>
    <pre>&lt;img src="https://example.com/sample.png" width="100" height="100" /></pre>
    <p>URLs can be relative or full.</p>
    <h3>Inline HTML</h3>
    <p>The editor supports a strict subset of HTML directly in code</p>
    <pre>To reboot your computer, press &lt;kbd>ctrl&lt;kbd>+&lt;kbd>alt&lt;kbd>+&lt;kbd>del&lt;kbd>.</pre>
    <p>Markdown is smart enough not to mangle your span-level HTML:</p>
    <pre>&lt;b>Markdown works *fine* in here.&lt;b></pre>
    <p>Block-level HTML elements have a few restrictions:</p>
    <ol>
        <li>They must be separated from surrounding text by blank lines.</li>
        <li>The begin and end tags of the outermost block element must not be indented.</li>
        <li>Markdown can't be used within HTML blocks.</li>
    </ol>
    <pre>&lt;pre>
    You can &lt;em>not&lt;em> use Markdown in here.
&lt;pre><pre></pre></pre>
    <h3>Syntax highlighting for code</h3>
    <p>
        Code blocks can be highlighted using <a
            href="https://github.com/highlightjs/highlight.js/"
            target="_blank">highlight.js</a
        >. Usually the highlight code will be automatically figure out the language but sometimes it
        may not.
    </p>
    <p>
        To manually specify the language of a fenced code block, add the language to the line with
        the opening fence:
    </p>
    <pre>```pythhon
import math

print('Hello world!')
```</pre>
    <h3>Tables</h3>
    <p>
        Our support for tables is based on the <a
            href="https://github.github.com/gfm/#tables-extension-"
            target="_blank">GitHub-flavored markdown table extension specification</a
        >.
    </p>
    <pre>| A header | Another header |
| -------- | -------------- |
| First    | row            |
| Second   | row            |</pre>
    <h3>Admonitions</h3>
    <p>
        We have three admonitionns note, tip and warning; these generate HTML for the admonitions
        and graphics. Use the UI(top bar above editor) to generate these.
    </p>
    <h3>Math Support</h3>
    <p>
        LaTeX math is supported via <a href="https://www.mathjax.org/" target="_blank">MathJax 4</a
        >. Use <code>$...$</code> for inline math and <code>$$...$$</code> (or
        <code>\[...\]</code>) for display math. Right now math is enabled on question pages only, not
        the ask page.
    </p>
    <p>
        Below are common formulas you can copy. Each block shows the LaTeX source; wrap inline
        expressions in single dollars and display expressions in double dollars or brackets.
    </p>
    {#each mathExamples as { title, code, render }}
        <h4>{title}</h4>
        <!-- Rendered output (MathJax typesets this; it is NOT inside <pre>). -->
        <div class="math-render">{render}</div>
        <!-- Copy-paste source (MathJax skips <pre> content by default). -->
        <pre><code>{code}</code></pre>
    {/each}
</main>

<style>
    h3 {
        margin-top: 10px;
        color: #094;
    }
    h4 {
        margin-top: 16px;
        color: #0a6c4f;
    }
    /* Live-rendered math preview (outside <pre>), above the copy-paste source. */
    .math-render {
        padding: 10px 12px;
        margin-top: 8px;
        background-color: #f6f9fb;
        border: 1px solid #dfe6ec;
        border-radius: 4px;
        overflow-x: auto;
    }
    main {
        max-width: 800px;
        margin: auto;
    }
    pre {
        background-color: #ddd;
        margin-bottom: 0;
        padding: 5px;
    }
    p {
        margin-top: 1rem;
        margin-bottom: 1rem;
    }
</style>
