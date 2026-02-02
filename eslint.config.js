module.exports = {
  root: true,
  files: ["./src/**/*.ts", "./src/**/*.js"],
  ignores: ["./*.config.js"],
  parser: "@typescript-eslint/parser",
  env: {
    browser: true,
    node: true,
    es6: true,
  },
  extends: [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:vitest/recommended",
  ],
  plugins: ["simple-import-sort", "promise", "vitest"],
  rules: {
    "no-use-before-define": "off",
    "import/no-anonymous-default-export": "error",
    "@typescript-eslint/no-use-before-define": "error",
    "no-void": ["error", { allowAsStatement: true }],
    "@typescript-eslint/no-unused-vars": [
      "error",
      {
        vars: "all",
        args: "none",
        argsIgnorePattern: "_",
        ignoreRestSiblings: false,
        varsIgnorePattern: "_",
      },
    ],
    "@typescript-eslint/no-var-requires": "off",
    "@typescript-eslint/typedef": "error",
    "promise/always-return": "off",
  },
  overrides: [
    {
      files: ["*.js"],
      rules: {
        "@typescript-eslint/explicit-module-boundary-types": "off",
        "@typescript-eslint/explicit-function-return-type": "off",
        "@typescript-eslint/ban-types": "off",
      },
    },
  ],
  parserOptions: {
    ecmaVersion: "latest",
  },
};
