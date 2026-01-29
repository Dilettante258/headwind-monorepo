import { readdir, readFile } from "node:fs/promises";
import { join } from "node:path";

export interface TestCase {
  name: string;
  input: string;
  expectedOutput: string;
}

export async function loadTestCases(casesDir: string): Promise<TestCase[]> {
  const cases: TestCase[] = [];
  const entries = await readdir(casesDir, { withFileTypes: true });

  for (const entry of entries) {
    if (entry.isDirectory()) {
      const caseDir = join(casesDir, entry.name);
      const input = await readFile(join(caseDir, "input.ts"), "utf-8");
      const expectedOutput = await readFile(join(caseDir, "output.ts"), "utf-8");

      cases.push({
        name: entry.name,
        input,
        expectedOutput,
      });
    }
  }

  return cases;
}
