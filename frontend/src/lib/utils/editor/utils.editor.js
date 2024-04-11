import DOMPurify from 'dompurify';
import { marked } from 'marked';
import hljs from 'highlight.js';

/**
 * Set the cursor position of input or textarea element
 * @file $lib/utils/editor/editor.utils.ts
 * @param { HTMLTextAreaElement } ctrl - The input or textarea element
 * @param {number} startPos - The start of the cursor position
 * @param {number} endPos - The end of the cursor position
 */
const setCaretPosition = (ctrl, startPos, endPos) => {
	if (ctrl.setSelectionRange) {
		ctrl.focus();
		ctrl.setSelectionRange(startPos, endPos);
	}
};

/**
 * Get the cursor position of input or textarea element
 * @file $lib/utils/editor/editor.utils.ts
 * @param { HTMLTextAreaElement } ctrl - The input or textarea element
 * @returns {Record<string, number>} The start and end position of the cursor in the input or textarea element
 */
export const getCaretPosition = (ctrl) =>
	ctrl.selectionStart
		? {
				start: ctrl.selectionStart,
				end: ctrl.selectionEnd
		  }
		: {
				start: 0,
				end: 0
		  };

/**
 * Parses markdown to HTML using `marked` and `sanitizes` the HTML using `DOMPurify`.
 * @file $lib/utils/editor/editor.utils.ts
 * @param { string } text - The markdown text to be parsed
 * @returns {string} The parsed markdown
 */
export const parseMarkdown = (text) => {
	marked.setOptions({
		renderer: new marked.Renderer(),
		highlight: function (code, lang) {
			const language = hljs.getLanguage(lang) ? lang : 'plaintext';
			return hljs.highlight(code, { language }).value;
		},
		langPrefix: 'hljs language-', // highlight.js css expects a top-level 'hljs' class.
		pedantic: false,
		gfm: true,
		breaks: false,
		sanitize: false,
		smartypants: false,
		xhtml: false
	});

	return DOMPurify.sanitize(marked.parse(text));
};

/**
 * Insert text into input or textarea element at a specified position.
 * @file $lib/utils/editor/editor.utils.ts
 * @param { HTMLTextAreaElement } contentTextArea - The input or textarea element
 * @param { string } text - The text to be inserted
 * @param { number } posStart - The start position where cursor should be placed
 * @param { number } posEnd - The end position where cursor should be placed
 */
const updateTexareaValue = (
	contentTextArea,
	text,
	posStart,
	posEnd = 0
) => {
	if (contentTextArea) {
		const start = contentTextArea.selectionStart + posStart;
		let end = start + posEnd;
		if (posEnd === 0) {
			end = contentTextArea.selectionStart + posStart;
		}
		contentTextArea.value = `${contentTextArea.value.slice(
			0,
			contentTextArea.selectionStart
		)}${text}${contentTextArea.value.slice(contentTextArea.selectionStart)}`;
		// console.log(`Test: ${text}, Start: ${start}, End: ${end}`);
		setCaretPosition(contentTextArea, start, end);
	}
};
/**
 *  Enable the usage of keyboard combinations to insert text into input or textarea element
 * @file $lib/utils/editor/editor.utils.ts
 * @param { Event } event - Event object
 * @param { HTMLTextAreaElement } contentTextArea - The input or textarea element
 */
export const useKeyCombinations = (event, contentTextArea) => {
	const keysPressed = {};
	event.target?.addEventListener('keydown', (e) => {
		const keyEvent = e;
		keysPressed[keyEvent.key] = true;

		if (
			(keysPressed['Control'] || keysPressed['Meta'] || keysPressed['Shift']) &&
			keyEvent.key == 'b'
		) {
			updateTexareaValue(contentTextArea, `****`, `****`.length / 2);
		} else if (
			(keysPressed['Control'] || keysPressed['Meta'] || keysPressed['Shift']) &&
			keyEvent.key == 'i'
		) {
			updateTexareaValue(contentTextArea, `**`, `**`.length / 2);
		} else if (
			(keysPressed['Control'] || keysPressed['Meta'] || keysPressed['Shift']) &&
			keyEvent.key === 'k'
		) {
			updateTexareaValue(contentTextArea, `[text](link)`, 1, `text`.length);
		}
	});

	event.target?.addEventListener('keyup', (e) => {
		delete keysPressed[(e).key];
	});
};

/**
 * Add markdown bold command to input or textarea element. Remove if present.
 * @file $lib/utils/editor/editor.utils.ts
 * @param { HTMLTextAreaElement } contentTextArea - The input or textarea element
 */
export const addBoldCommand = (contentTextArea) => {
	const text = `****`;
	if (contentTextArea.value.indexOf(text) !== -1) {
		contentTextArea.value = contentTextArea.value.replace(text, '');
	} else {
		updateTexareaValue(contentTextArea, text, text.length / 2);
	}
};
/**
 * Add markdown italics command to input or textarea element. Remove if present.
 * @file $lib/utils/editor/editor.utils.ts
 * @param { HTMLTextAreaElement } contentTextArea - The input or textarea element
 */
export const addItalicCommand = (contentTextArea) => {
	if (contentTextArea.value.indexOf(`*emphasize text*`) !== -1) {
		contentTextArea.value = contentTextArea.value.replace(`*emphasize text*`, '');
	} else {
		updateTexareaValue(contentTextArea, `*emphasize text*`, 1, `emphasize text`.length);
	}
};
/**
 * Add markdown link command to input or textarea element. Remove if present.
 * @file $lib/utils/editor/editor.utils.ts
 * @param { HTMLTextAreaElement } contentTextArea - The input or textarea element
 */
export const addLinkCommand = (contentTextArea) => {
	if (contentTextArea.value.indexOf(`[text](link)`) !== -1) {
		contentTextArea.value = contentTextArea.value.replace(`[text](link)`, '');
	} else {
		updateTexareaValue(contentTextArea, `[text](link)`, 1, `text`.length);
	}
};
/**
 * Add markdown unordered list command to input or textarea element. Remove if present.
 * @file $lib/utils/editor/editor.utils.ts
 * @param { HTMLTextAreaElement } contentTextArea - The input or textarea element
 */
export const addUnorderedListCommand = (contentTextArea) => {
	if (contentTextArea.value.indexOf(`\n- First item\n- Second item\n`) !== -1) {
		contentTextArea.value = contentTextArea.value.replace(`\n- First item\n- Second item\n`, '');
	} else {
		updateTexareaValue(
			contentTextArea,
			`\n- First item\n- Second item\n`,
			`\n- `.length,
			`First item`.length
		);
	}
};
/**
 * Add markdown ordered list command to input or textarea element. Remove if present.
 * @file $lib/utils/editor/editor.utils.ts
 * @param { HTMLTextAreaElement } contentTextArea - The input or textarea element
 */
export const addOrderedListCommand = (contentTextArea) => {
	if (contentTextArea.value.indexOf(`\n1. First item\n2. Second item\n`) !== -1) {
		contentTextArea.value = contentTextArea.value.replace(`\n1. First item\n2. Second item\n`, '');
	} else {
		updateTexareaValue(
			contentTextArea,
			`\n1. First item\n2. Second item\n`,
			`\n1. `.length,
			`First item`.length
		);
	}
};
/**
 * Add markdown heading two command to input or textarea element. Remove if present.
 * @file $lib/utils/editor/editor.utils.ts
 * @param { HTMLTextAreaElement } contentTextArea - The input or textarea element
 */
export const addHeadingTwoCommand = (contentTextArea) => {
	if (contentTextArea.value.indexOf(`\n## Your heading two \n\n`) !== -1) {
		contentTextArea.value = contentTextArea.value.replace(`\n## Your heading two \n\n`, '');
	} else {
		updateTexareaValue(
			contentTextArea,
			`\n## Your heading two \n\n`,
			`\n## `.length,
			`Your heading two `.length
		);
	}
};
/**
 * Add markdown heading three command to input or textarea element. Remove if present.
 * @file $lib/utils/editor/editor.utils.ts
 * @param { HTMLTextAreaElement } contentTextArea - The input or textarea element
 */
export const addHeadingThreeCommand = (contentTextArea) => {
	if (contentTextArea.value.indexOf(`\n### Your heading three \n\n`) !== -1) {
		contentTextArea.value = contentTextArea.value.replace(`\n### Your heading three \n\n`, '');
	} else {
		updateTexareaValue(
			contentTextArea,
			`\n### Your heading three \n\n`,
			`\n### `.length,
			`Your heading three `.length
		);
	}
};

/**
 * Add markdown heading four command to input or textarea element. Remove if present.
 * @file $lib/utils/editor/editor.utils.ts
 * @param { HTMLTextAreaElement } contentTextArea - The input or textarea element
 */
export const addHeadingFourCommand = (contentTextArea) => {
	if (contentTextArea.value.indexOf(`\n#### Your heading four \n\n`) !== -1) {
		contentTextArea.value = contentTextArea.value.replace(`\n#### Your heading four \n\n`, '');
	} else {
		updateTexareaValue(
			contentTextArea,
			`\n#### Your heading four \n\n`,
			`\n#### `.length,
			`Your heading four `.length
		);
	}
};
/**
 * Add markdown heading five command to input or textarea element. Remove if present.
 * @file $lib/utils/editor/editor.utils.ts
 * @param { HTMLTextAreaElement } contentTextArea - The input or textarea element
 */
export const addHeadingFiveCommand = (contentTextArea) => {
	if (contentTextArea.value.indexOf(`\n##### Your heading five \n\n`) !== -1) {
		contentTextArea.value = contentTextArea.value.replace(`\n##### Your heading five \n\n`, '');
	} else {
		updateTexareaValue(
			contentTextArea,
			`\n##### Your heading five \n\n`,
			`\n##### `.length,
			`Your heading five `.length
		);
	}
};

/**
 * Add markdown heading six command to input or textarea element. Remove if present.
 * @file $lib/utils/editor/editor.utils.ts
 * @param { HTMLTextAreaElement } contentTextArea - The input or textarea element
 */
export const addHeadingSixCommand = (contentTextArea) => {
	if (contentTextArea.value.indexOf(`\n###### Your heading six \n\n`) !== -1) {
		contentTextArea.value = contentTextArea.value.replace(`\n###### Your heading six \n\n`, '');
	} else {
		updateTexareaValue(
			contentTextArea,
			`\n###### Your heading six \n\n`,
			`\n###### `.length,
			`Your heading six `.length
		);
	}
};

/**
 * Add markdown image command to input or textarea element. Remove if present.
 * @file $lib/utils/editor/editor.utils.ts
 * @param { HTMLTextAreaElement } contentTextArea - The input or textarea element
 */
export const addImageCommand = (contentTextArea) => {
	if (contentTextArea.value.indexOf(`![alt text](url)`) !== -1) {
		contentTextArea.value = contentTextArea.value.replace(`![alt text](url)`, '');
	} else {
		updateTexareaValue(contentTextArea, `![alt text](url)`, `![`.length, `alt text`.length);
	}
};

/**
 * Add markdown codeblock command to input or textarea element. Remove if present.
 * @file $lib/utils/editor/editor.utils.ts
 * @param { HTMLTextAreaElement } contentTextArea - The input or textarea element
 */
export const addCodeBlockCommand = (contentTextArea) => {
	if (contentTextArea.value.indexOf('\n```language\n<code here>\n```') !== -1) {
		contentTextArea.value = contentTextArea.value.replace('\n```language\n<code here>\n```', '');
	} else {
		updateTexareaValue(
			contentTextArea,
			'\n```language\n<code here>\n```',
			'\n```'.length,
			`language\n<code here>`.length
		);
	}
};

/**
 * Add markdown notice command to input or textarea element. Remove if present.
 * @file $lib/utils/editor/editor.utils.ts
 * @param { HTMLTextAreaElement } contentTextArea - The input or textarea element
 */
export const addNoteCommand = (contentTextArea) => {
	if (
		contentTextArea.value.indexOf(
			'\n<div class="admonition note">\n<span class="title"><b>Note:</b></span>\n<p></p>\n</div>'
		) !== -1
	) {
		contentTextArea.value = contentTextArea.value.replace(
			'\n<div class="admonition note">\n<span class="title"><b>Note:</b></span>\n<p></p>\n</div>',
			''
		);
	} else {
		updateTexareaValue(
			contentTextArea,
			'\n<div class="admonition note">\n<span class="title"><b>Note:</b></span>\n<p></p>\n</div>',
			'\n<div class="admonition note">\n<span class="title"><b>Note:</b>'.length
		);
	}
};

/**
 * Add markdown tip command to input or textarea element. Remove if present.
 * @file $lib/utils/editor/editor.utils.ts
 * @param { HTMLTextAreaElement } contentTextArea - The input or textarea element
 */
export const addTipCommand = (contentTextArea) => {
	if (
		contentTextArea.value.indexOf(
			'\n<div class="admonition tip">\n<span class="title"><b>Tip:</b></span>\n<p></p>\n</div>'
		) !== -1
	) {
		contentTextArea.value = contentTextArea.value.replace(
			'\n<div class="admonition tip">\n<span class="title"><b>Tip:</b></span>\n<p></p>\n</div>',
			''
		);
	} else {
		updateTexareaValue(
			contentTextArea,
			'\n<div class="admonition tip">\n<span class="title"><b>Tip:</b></span>\n<p></p>\n</div>',
			'\n<div class="admonition tip">\n<span class="title"><b>Tip:</b>'.length
		);
	}
};

/**
 * Add markdown warning command to input or textarea element. Remove if present.
 * @file $lib/utils/editor/editor.utils.ts
 * @param { HTMLTextAreaElement } contentTextArea - The input or textarea element
 */
export const addWarningCommand = (contentTextArea) => {
	if (
		contentTextArea.value.indexOf(
			'\n<div class="admonition warning">\n<span class="title"><b>Warning:</b></span>\n<p></p>\n</div>'
		) !== -1
	) {
		contentTextArea.value = contentTextArea.value.replace(
			'\n<div class="admonition warning">\n<span class="title"><b>Warning:</b></span>\n<p></p>\n</div>',
			''
		);
	} else {
		updateTexareaValue(
			contentTextArea,
			'\n<div class="admonition warning">\n<span class="title"><b>Warning:</b></span>\n<p></p>\n</div>',
			'\n<div class="admonition warning">\n<span class="title"><b>Warning:</b>'.length
		);
	}
};