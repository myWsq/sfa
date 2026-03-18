import schema from "./lib/config/schema.json" assert { type: "json" };
{{IMPORT_BLOCK}}

export const packageName = "{{PACKAGE_NAME}}";
export const packageDepth = {{PACKAGE_DEPTH}};
export const packageIndex = {{PACKAGE_INDEX}};
export const dependencyNames = [{{DEPENDENCY_NAME_ARRAY}}];

export function describePackage() {
  return {
    packageName,
    packageDepth,
    packageIndex,
    dependencyNames,
    schemaVersion: schema.$id,
  };
}

export function inspectDependencies() {
  return dependencyNames.map((name, index) => ({
    name,
    index,
    seen: Boolean(schema.properties[`dependency_${index}`]),
  }));
}
