"use strict";

const descriptor = {
  name: "{{PACKAGE_NAME}}",
  depth: {{PACKAGE_DEPTH}},
  index: {{PACKAGE_INDEX}},
};

exports.loadRuntimeState = function loadRuntimeState(env) {
  const source = env || process.env;
  return {
    name: descriptor.name,
    depth: descriptor.depth,
    index: descriptor.index,
    nodeEnv: source.NODE_ENV || "development",
  };
};

exports.listDependencyExports = function listDependencyExports() {
  return [{{DEPENDENCY_NAME_ARRAY}}].map((name) => ({
    name,
    entry: `${name}/dist/index.cjs`,
  }));
};
