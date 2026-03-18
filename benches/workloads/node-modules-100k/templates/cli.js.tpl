#!/usr/bin/env node

const args = process.argv.slice(2);
const flags = new Set(args);

if (flags.has("--build")) {
  process.stdout.write("building {{PACKAGE_NAME}}\\n");
} else if (flags.has("--test")) {
  process.stdout.write("testing {{PACKAGE_NAME}}\\n");
} else {
  process.stdout.write(
    JSON.stringify({
      packageName: "{{PACKAGE_NAME}}",
      packageDepth: {{PACKAGE_DEPTH}},
      packageIndex: {{PACKAGE_INDEX}},
      dependencies: [{{DEPENDENCY_NAME_ARRAY}}],
    }) + "\\n"
  );
}
