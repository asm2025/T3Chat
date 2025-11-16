// Lightweight global fetch interceptor to surface HTML error pages as errors
// and standardize JSON parsing errors across the app.

type OriginalFetch = typeof window.fetch;

let installed = false;

export function installFetchInterceptor(): void {
	if (installed || typeof window === "undefined" || !window.fetch) return;
	installed = true;

	const originalFetch: OriginalFetch = window.fetch.bind(window);

	window.fetch = async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
		// Do NOT enforce content-type or status here.
		// Leave handling to higher-level helpers (e.g., fetchJson) so callers can declare expectations.
		return originalFetch(input, init);
	};
}

// Preferred global helper for JSON APIs. Centralizes error handling.
export async function fetchJson<T>(input: RequestInfo | URL, init?: RequestInit): Promise<T> {
	const res = await fetch(input, init);
	const contentType = res.headers.get("content-type") || "";
	// If the response is not OK, try to read JSON error first, else fallback to text snippet.
	if (!res.ok) {
		let message = `Request failed ${res.status}`;
		if (contentType.includes("application/json")) {
			try {
				const data = await res.json();
				const errorText = typeof data === "string" ? data : JSON.stringify(data);
				throw new Error(`${message}: ${errorText}`);
			} catch {
				// fallthrough to text
			}
		}
		const text = await res.text().catch(() => "");
		const snippet = text.slice(0, 200);
		throw new Error(`${message}: ${snippet}`);
	}

	// Enforce JSON when caller expects JSON
	if (!contentType.includes("application/json")) {
		const text = await res.text().catch(() => "");
		const snippet = text.slice(0, 200);
		throw new SyntaxError(`Unexpected content-type "${contentType}". ${snippet}`);
	}

	return (await res.json()) as T;
}


