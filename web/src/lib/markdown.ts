// Statement and board Markdown with KaTeX math.
//
// Raw HTML in the source is escaped rather than rendered, and link targets
// are limited to http(s), mailto and relative URLs, so user content cannot
// inject markup or script.

import { Marked, type Tokens } from 'marked';
import katex from 'katex';

function escapeHtml(s: string): string {
	return s.replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!);
}

function safeUrl(href: string): string | null {
	const h = href.trim();
	if (/^(https?:|mailto:)/i.test(h) || /^[/#.]/.test(h) || !/^[a-z][a-z0-9+.-]*:/i.test(h)) return h;
	return null;
}

function tex(src: string, display: boolean): string {
	return katex.renderToString(src, { displayMode: display, throwOnError: false, output: 'html', strict: 'ignore', trust: false });
}

const marked = new Marked({ gfm: true, breaks: false });

marked.use({
	extensions: [
		{
			name: 'blockMath',
			level: 'block',
			start: (src: string) => src.match(/^\$\$/m)?.index,
			tokenizer(src: string) {
				const m = /^\$\$([\s\S]+?)\$\$(?:\n|$)/.exec(src);
				if (m) return { type: 'blockMath', raw: m[0], text: m[1].trim() };
			},
			renderer: (t) => `<div class="math-block">${tex((t as Tokens.Generic).text, true)}</div>`
		},
		{
			name: 'inlineMath',
			level: 'inline',
			start: (src: string) => src.indexOf('$'),
			tokenizer(src: string) {
				const m = /^\$(?!\s)((?:\\\$|[^$\n])+?)(?<!\s)\$/.exec(src);
				if (m) return { type: 'inlineMath', raw: m[0], text: m[1] };
			},
			renderer: (t) => tex((t as Tokens.Generic).text, false)
		}
	],
	renderer: {
		html(token: Tokens.HTML | Tokens.Tag) {
			return escapeHtml(token.text);
		},
		link({ href, title, tokens }: Tokens.Link) {
			const text = this.parser.parseInline(tokens);
			const url = safeUrl(href);
			if (!url) return text;
			const t = title ? ` title="${escapeHtml(title)}"` : '';
			return `<a href="${escapeHtml(url)}"${t} rel="nofollow noopener" target="_blank">${text}</a>`;
		},
		image({ href, title, text }: Tokens.Image) {
			const url = safeUrl(href);
			if (!url) return escapeHtml(text);
			const t = title ? ` title="${escapeHtml(title)}"` : '';
			return `<img src="${escapeHtml(url)}" alt="${escapeHtml(text)}"${t} loading="lazy">`;
		}
	}
});

export function renderMarkdown(src: string | null | undefined): string {
	if (!src) return '';
	return marked.parse(src, { async: false }) as string;
}
