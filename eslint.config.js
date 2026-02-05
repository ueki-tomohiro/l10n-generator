const tsParser = require("@typescript-eslint/parser");
const tsPlugin = require("@typescript-eslint/eslint-plugin");
const simpleImportSort = require("eslint-plugin-simple-import-sort");
const promisePlugin = require("eslint-plugin-promise");
const vitestPlugin = require("eslint-plugin-vitest");

module.exports = [
  {
    ignores: ["./*.config.js", "lib/**/*", "node_modules/**/*", "coverage/**/*"],
  },
  {
    files: ["src/**/*.ts", "src/**/*.js"],
    languageOptions: {
      parser: tsParser,
      ecmaVersion: "latest",
      sourceType: "module",
      globals: {
        console: "readonly",
        process: "readonly",
        __dirname: "readonly",
        __filename: "readonly",
        Buffer: "readonly",
        module: "readonly",
        require: "readonly",
        exports: "readonly",
      },
    },
    plugins: {
      "@typescript-eslint": tsPlugin,
      "simple-import-sort": simpleImportSort,
      promise: promisePlugin,
      vitest: vitestPlugin,
    },
    rules: {
      ...tsPlugin.configs.recommended.rules,
      ...vitestPlugin.configs.recommended.rules,
      "no-use-before-define": "off",
      "@typescript-eslint/no-use-before-define": "error",
      "no-void": ["error", { allowAsStatement: true }],
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          vars: "all",
          args: "none",
          argsIgnorePattern: "_",
          ignoreRestSiblings: false,
          varsIgnorePattern: "^(_|locales$|credentialTypes$|fileTypes$|outputTypes$)",
        },
      ],
      "@typescript-eslint/no-var-requires": "off",
      "@typescript-eslint/no-explicit-any": "warn",
      "@typescript-eslint/typedef": "error",
      "promise/always-return": "off",
    },
  },
  {
    files: ["src/**/*.js"],
    rules: {
      "@typescript-eslint/explicit-module-boundary-types": "off",
      "@typescript-eslint/explicit-function-return-type": "off",
      "@typescript-eslint/ban-types": "off",
    },
  },
];
