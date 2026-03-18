const descriptor = {
  name: "{{PACKAGE_NAME}}",
  depth: {{PACKAGE_DEPTH}},
  index: {{PACKAGE_INDEX}},
  marker: "synthetic-benchmark-package",
};

export function loadRuntimeState(env = process.env) {
  return {
    ...descriptor,
    cwd: env.PWD || "",
    shell: env.SHELL || "",
    nodeEnv: env.NODE_ENV || "development",
  };
}

export function listDependencyExports() {
  return [{{DEPENDENCY_NAME_ARRAY}}].map((name) => ({
    name,
    entry: `${name}/dist/index.js`,
  }));
}
