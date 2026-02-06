export async function fetchSemanticNames(
  elementTree: string,
  baseUrl = '',
): Promise<Record<string, string>> {
  const url = `${baseUrl}/api/semantic-names`;
  const res = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ elementTree }),
  });

  if (!res.ok) {
    const body = await res.json().catch(() => null);
    throw new Error(
      (body as any)?.error ?? `API request failed (${res.status})`,
    );
  }

  const data = (await res.json()) as { names: Record<string, string> };
  return data.names;
}
