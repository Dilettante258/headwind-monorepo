const SYSTEM_PROMPT = `You are a CSS class naming expert. Given an element tree of a web component, generate semantic CSS class names for elements that have Tailwind utility classes.

Rules:
1. Use kebab-case (e.g. "page-header", "nav-link", "card-title").
2. Names should describe the element's PURPOSE or ROLE in the component, NOT its visual properties.
3. Keep names concise: 1-3 words joined by hyphens, max 30 characters.
4. All names must be unique within the response.
5. Must be valid CSS class names: start with a letter, only lowercase letters, digits, and hyphens.
6. Only generate names for elements that have CSS classes (shown between the tag name and [ref=eN]).
7. Skip elements that have no classes (e.g. "- div [ref=e4]" or "- p: some text [ref=e3]").
8. Use the component name (## header), tag name, text content, and tree position as context.
9. Respond with ONLY a JSON object mapping ref IDs to semantic names. No markdown, no explanation.`;

function corsHeaders(origin: string | null): Record<string, string> {
	return {
		'Access-Control-Allow-Origin': origin ?? '*',
		'Access-Control-Allow-Methods': 'POST, OPTIONS',
		'Access-Control-Allow-Headers': 'Content-Type',
		'Access-Control-Max-Age': '86400',
	};
}

function jsonResponse(
	body: unknown,
	status: number,
	headers: Record<string, string>,
): Response {
	return new Response(JSON.stringify(body), {
		status,
		headers: { ...headers, 'Content-Type': 'application/json' },
	});
}

/** Strip markdown fences and extract JSON object from AI text */
function extractJson(text: string): string {
	let s = text.trim();
	const fenceMatch = s.match(/```(?:json)?\s*\n?([\s\S]*?)\n?\s*```/);
	if (fenceMatch) s = fenceMatch[1].trim();
	const braceStart = s.indexOf('{');
	const braceEnd = s.lastIndexOf('}');
	if (braceStart !== -1 && braceEnd > braceStart) {
		s = s.slice(braceStart, braceEnd + 1);
	}
	return s;
}

/** Validate and sanitize AI-generated class names */
function sanitizeNames(raw: Record<string, unknown>): Record<string, string> {
	const result: Record<string, string> = {};
	const used = new Set<string>();
	const validPattern = /^[a-z][a-z0-9-]*$/;

	for (const [ref, value] of Object.entries(raw)) {
		if (typeof value !== 'string' || !ref.match(/^e\d+$/)) continue;

		let name = value
			.toLowerCase()
			.replace(/[^a-z0-9-]/g, '-')
			.replace(/-+/g, '-')
			.replace(/^-|-$/g, '');

		if (!name || !/^[a-z]/.test(name)) name = 'el-' + name;
		if (name.length > 30) name = name.slice(0, 30).replace(/-$/, '');
		if (!validPattern.test(name)) continue;

		// Deduplicate
		if (used.has(name)) {
			let suffix = 2;
			while (used.has(`${name}-${suffix}`)) suffix++;
			name = `${name}-${suffix}`;
		}
		used.add(name);
		result[ref] = name;
	}

	return result;
}

async function handleSemanticNames(
	request: Request,
	env: Env,
	headers: Record<string, string>,
): Promise<Response> {
	let body: { elementTree?: string };
	try {
		body = (await request.json()) as { elementTree?: string };
	} catch {
		return jsonResponse({ error: 'Invalid JSON body' }, 400, headers);
	}

	if (!body.elementTree || typeof body.elementTree !== 'string') {
		return jsonResponse({ error: 'Missing or invalid "elementTree" field' }, 400, headers);
	}

	const userPrompt = `Element tree:\n${body.elementTree}\n\nGenerate a JSON object mapping each ref ID (e.g. "e1") to a semantic class name. Only include refs that have CSS classes. Example: {"e1": "page-header", "e2": "nav-link"}`;

	try {
		const response = (await env.AI.run('@cf/meta/llama-3.1-8b-instruct-fp8', {
			messages: [
				{ role: 'system', content: SYSTEM_PROMPT },
				{ role: 'user', content: userPrompt },
			],
			max_tokens: 1024,
			temperature: 0.3,
		})) as { response?: string };

		const aiText = response?.response;
		if (!aiText) {
			return jsonResponse({ error: 'AI returned empty response' }, 502, headers);
		}

		const jsonStr = extractJson(aiText);
		let parsed: unknown;
		try {
			parsed = JSON.parse(jsonStr);
		} catch {
			return jsonResponse(
				{ error: 'Failed to parse AI response as JSON', raw: aiText.slice(0, 500) },
				502,
				headers,
			);
		}

		if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
			return jsonResponse({ error: 'AI response is not a JSON object' }, 502, headers);
		}

		const names = sanitizeNames(parsed as Record<string, unknown>);
		return jsonResponse({ names }, 200, headers);
	} catch (err) {
		const message = err instanceof Error ? err.message : String(err);
		return jsonResponse({ error: `AI processing failed: ${message}` }, 502, headers);
	}
}

export default {
	async fetch(request, env, ctx): Promise<Response> {
		const origin = request.headers.get('Origin');
		const headers = corsHeaders(origin);

		if (request.method === 'OPTIONS') {
			return new Response(null, { status: 204, headers });
		}

		const url = new URL(request.url);

		if (url.pathname === '/api/semantic-names' && request.method === 'POST') {
			return handleSemanticNames(request, env, headers);
		}

		return jsonResponse({ error: 'Not Found' }, 404, headers);
	},
} satisfies ExportedHandler<Env>;
