import { transformJsx, transformHtml } from "../../../target/pkg-node/headwind_wasm.js";
import assert from "node:assert";

let passed = 0;

// Test 1: JSX transform - Global mode
{
  const result = transformJsx(
    `export default function App() {
      return <div className="p-4 text-center">Hello</div>;
    }`,
    "App.tsx",
    { namingMode: "hash", outputMode: { type: "global" } }
  );

  assert.ok(result.code.length > 0, "code should not be empty");
  assert.ok(result.css.includes("padding"), "css should contain padding");
  assert.ok(result.css.includes("text-align"), "css should contain text-align");
  assert.ok(!result.code.includes("p-4 text-center"), "should not contain original classes");
  assert.strictEqual(Object.keys(result.classMap).length, 1, "should have 1 class mapping");
  console.log("PASS: JSX transform - Global mode");
  passed++;
}

// Test 2: HTML transform
{
  const result = transformHtml(
    `<div class="p-4 text-center">Hello</div>`,
    {}
  );

  assert.ok(!result.code.includes("p-4 text-center"), "should not contain original classes");
  assert.ok(result.css.includes("padding"), "css should contain padding");
  assert.ok(result.css.includes("text-align"), "css should contain text-align");
  console.log("PASS: HTML transform");
  passed++;
}

// Test 3: CSS Modules + Dot access
{
  const result = transformJsx(
    `function App() { return <div className="p-4">Hi</div>; }`,
    "App.tsx",
    {
      namingMode: "camelCase",
      outputMode: { type: "cssModules", bindingName: "styles", access: "dot" },
    }
  );

  assert.ok(result.code.includes("import styles from"), "should have CSS modules import");
  assert.ok(result.code.includes("styles."), "should have styles dot access");
  assert.ok(!result.code.includes('styles["'), "should NOT have bracket access");
  console.log("PASS: CSS Modules + Dot access");
  passed++;
}

// Test 4: CSS Modules + Bracket access
{
  const result = transformJsx(
    `function App() { return <div className="p-4 m-2">Hi</div>; }`,
    "App.tsx",
    {
      namingMode: "hash",
      outputMode: { type: "cssModules", access: "bracket" },
    }
  );

  assert.ok(result.code.includes("import styles from"), "should have CSS modules import");
  assert.ok(result.code.includes('styles["'), "should have bracket access");
  console.log("PASS: CSS Modules + Bracket access");
  passed++;
}

// Test 5: CamelCase naming in Global mode
{
  const result = transformJsx(
    `function App() { return <div className="text-center bg-blue-500">Hi</div>; }`,
    "App.jsx",
    { namingMode: "camelCase", outputMode: { type: "global" } }
  );

  const className = Object.values(result.classMap)[0];
  assert.ok(!className.includes("_"), "camelCase name should not contain underscore");
  assert.ok(!className.includes("-"), "camelCase name should not contain hyphen");
  console.log("PASS: CamelCase naming in Global mode");
  passed++;
}

// Test 6: Readable naming
{
  const result = transformJsx(
    `function App() { return <div className="p-4 m-2">Hi</div>; }`,
    "App.jsx",
    { namingMode: "readable", outputMode: { type: "global" } }
  );

  const className = Object.values(result.classMap)[0];
  assert.strictEqual(className, "p4_m2", "readable name should be p4_m2");
  console.log("PASS: Readable naming");
  passed++;
}

// Test 7: Default options (undefined)
{
  const result = transformHtml(`<div class="m-2">Hello</div>`);
  assert.ok(result.css.includes("margin"), "css should contain margin");
  console.log("PASS: Default options (undefined)");
  passed++;
}

// Test 8: Multiple elements with class reuse
{
  const result = transformJsx(
    `function App() {
      return (
        <div>
          <p className="p-4 m-2">A</p>
          <p className="p-4 m-2">B</p>
          <span className="text-red-500">C</span>
        </div>
      );
    }`,
    "App.jsx",
    {}
  );

  assert.strictEqual(Object.keys(result.classMap).length, 2, "should have 2 unique class mappings");
  console.log("PASS: Multiple elements with class reuse");
  passed++;
}

// Test 9: CSS Modules custom binding name
{
  const result = transformJsx(
    `function App() { return <div className="p-4">Hi</div>; }`,
    "App.tsx",
    {
      outputMode: { type: "cssModules", bindingName: "css" },
    }
  );

  assert.ok(result.code.includes("import css from"), "should use custom binding name");
  assert.ok(result.code.includes("css."), "should use custom binding in access");
  console.log("PASS: CSS Modules custom binding name");
  passed++;
}

// Test 10: CSS Modules custom import path
{
  const result = transformJsx(
    `function App() { return <div className="p-4">Hi</div>; }`,
    "App.tsx",
    {
      outputMode: { type: "cssModules", importPath: "../styles/app.module.css" },
    }
  );

  assert.ok(
    result.code.includes("../styles/app.module.css"),
    "should use custom import path"
  );
  console.log("PASS: CSS Modules custom import path");
  passed++;
}

console.log(`\n${passed}/${passed} tests passed!`);
