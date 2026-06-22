<svelte:head>
	<title>Markdown Help ❤ Kunjika</title>
</svelte:head>
<main style="margin-top:20px">
	<h2>Introduction</h2>
	<p>
		Markdown is a simple format popularized by Stackoverflow. You can read original stuff at
		<a href="https://daringfireball.net/linked/2014/01/08/markdown-extension"
			>Official Markdown Website</a
		>. Most popular variation of markdown is called CommonMark but there is also github flavoured
		markdown. We also use markdown for Kunjika.
	</p>
	<h3>Code and Preformatted Text</h3>
	<p>
		Indent four spaces to create an escaped <code>&lt;pre&gt;</code> and <code>&lt;code&gt;</code>
		block:
	</p>
	<pre>
<span style="background-color: #fafafa;margin-left:2px"></span>printf("%d\n", 42);  /* what was the
<span style="background-color: #fafafa;margin-left:2px"
		></span>                        question again? */</pre>
	<p style="margin-top:5px">
		The text will be wrapped in tags, and displayed in a monospaced font. The first four spaces will
		be stripped off, but all other whitespace will be preserved.
	</p>
	<p>Markdown and HTML are ignored within a code block:</p>
	<pre>
<span style="background-color: #fafafa;margin-left:2px"></span>&lt;blink&gt;
<span style="background-color: #fafafa;margin-left:2px"></span>You would hate this if it weren't
<span style="background-color: #fafafa;margin-left:2px"></span>wrapped in a code block.
<span style="background-color: #fafafa;margin-left:2px"></span>&lt;/blink&gt;</pre>
	<p>
		You can also create code blocks with fenced code consisting of 3 or more tildes followed by the
		language name optionally of the code block
	</p>
	<pre>```python
print("Hello world")
```</pre>
	<h3>Code Spans</h3>
	<p>
		For inline code you can use <code>`code`</code>. An exmaple would be <code>`make html`</code>.
	</p>
	<p>
		Like code blocks, code spans will be displayed in a monospaced font. Markdown and HTML will not
		work within them. Note that, unlike code blocks, code spans require you to manually escape any
		HTML within!
	</p>
	<p>
		If your code itself contains backticks, you may have to use multiple backticks as delimiters:
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
		The link definitions can appear anywhere in the document -- before or after the place where you
		use them. The link definition names <code>[1]</code> and <code>[yahoo]</code> can be any unique
		string, and are case-insensitive; <code>[yahoo]</code> is the same as <code>[YAHOO]</code>.
	</p>
	<p>You can also use standard HTML hyperlink syntax.</p>
	<pre>&lt;a href="https://example.com" title="example"&gt;example&lt;/a&gt;</pre>
	<p>
		The editor will automatically convert naked links to clickable URLs(in most but not all cases
		--- beware of unusal characters in your URLs)
	</p>
	<pre>I often visit https://kunjika.ashtavakra.org</pre>
	<h3>Header</h3>
	<p>Use Underline text to make the two <code>&lt;h1&gt;&lt;h2&gt;</code> top-level headers :</p>
	<pre>Header 1
========
Header 2
--------</pre>
	<p>
		The number of = or - signs doesn't matter; one will work. But using enough to underline the text
		makes your titles look better in plain text.
	</p>
	<p>Use hash marks for several levels of headers:</p>
	<pre># Header 1 #
## Header 2 ##
### Header 3 ###</pre>
	<p>The closing # characters are optional.</p>
	<h3>Horizontal Rules</h3>
	<p>
		Insert a horizontal rule <code>&lt;hr/&gt;</code> by putting three or more hyphens, asterisks, or
		underscores on a line by themselves:
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
		The word in square brackets is the alt text, which gets displayed if the browser can't show the
		image. Be sure to include meaningful alt text for screen-reading software.
	</p>

	<p>Just like links, images work with reference syntax and titles:</p>
	<pre>This page is ![valid XHTML][checkmark].

[checkmark]: https://w3.org/Icons/valid-xhtml10
             "What are you smiling at?"</pre>
	<p>
		You can also use standard HTML image syntax, which allows you to scale the width and height of
		the image.
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
		>. Usually the highlight code will be automatically figure out the language but sometimes it may
		not.
	</p>
	<p>
		To manually specify the language of a fenced code block, add the language to the line with the
		opening fence:
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
		We have three admonitionns note, tip and warning; these generate HTML for the admonitions and
		graphics. Use the UI(top bar above editor) to generate these.
	</p>
	<h3>Mmath Support</h3>
	<p>
		LaTeX math is supported via <a href="https://www.mathjax.org/" target="_blank">MathJax 2</a>.
		Use $$ for inline and \[\] for display math. Refer to LaTeX and MathJax for more details. Soon
		we will support MathJax 4. Right now this is not enabled for ask page but only for questions.
	</p>
</main>

<style>
	h3 {
		margin-top: 10px;
		color: #094;
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
