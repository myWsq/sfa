{
  "name": "{{PACKAGE_NAME}}",
  "version": "{{PACKAGE_VERSION}}",
  "description": "Synthetic benchmark package {{PACKAGE_NAME}} at dependency depth {{PACKAGE_DEPTH}}",
  "license": "MIT",
  "type": "module",
  "sideEffects": false,
  "main": "./dist/index.cjs",
  "module": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": {
      "types": "./dist/index.d.ts",
      "import": "./dist/index.js",
      "require": "./dist/index.cjs"
    },
    "./cli": "./src/cli.js"
  },
  "files": ["dist", "lib", "src", "index.js", "index.d.ts"],
  "scripts": {
    "build": "node ./src/cli.js --build",
    "test": "node ./src/cli.js --test"
  },
  "dependencies": {
{{DEPENDENCIES_JSON}}
  }
}
