// ESLint flat config for React + TypeScript + Vite
import js from "@eslint/js";
import tseslint from "typescript-eslint";
import react from "eslint-plugin-react";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import prettier from "eslint-config-prettier";

export default [
  // Ignore build and vendor outputs
  { ignores: ["dist/**", "node_modules/**", "public/pkg/**"] },

  // Browser globals for client code and public assets
  {
    files: ["src/**/*.{ts,tsx}", "public/**/*.js"],
    languageOptions: {
      globals: {
        window: "readonly",
        document: "readonly",
        navigator: "readonly",
        location: "readonly",
        ResizeObserver: "readonly",
        URLSearchParams: "readonly",
      },
    },
  },

  // Node globals for scripts and config
  {
    files: ["scripts/**/*.{js,mjs,ts}", "vite.config.ts"],
    languageOptions: {
      globals: {
        process: "readonly",
        console: "readonly",
      },
    },
  },

  // Base + TypeScript + React
  ...tseslint.config(js.configs.recommended, ...tseslint.configs.recommended, {
    files: ["**/*.{ts,tsx}", "**/*.{js,mjs}", "vite.config.ts"],
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
      parserOptions: { project: false },
    },
    plugins: {
      react,
      "react-hooks": reactHooks,
      "react-refresh": reactRefresh,
    },
    settings: { react: { version: "detect" } },
    rules: {
      // React 17+ with automatic runtime
      "react/react-in-jsx-scope": "off",
      // Not using PropTypes in TS
      "react/prop-types": "off",
      // Allow empty catch blocks used around localStorage access, etc.
      "no-empty": ["error", { allowEmptyCatch: true }],
      // General code-style & correctness
      eqeqeq: ["error", "smart"],
      curly: ["error", "multi-line"],
      yoda: ["error", "never"],
      "no-else-return": ["error", { allowElseIf: false }],
      "no-nested-ternary": "error",
      "no-implicit-coercion": [
        "error",
        { boolean: true, number: true, string: true, disallowTemplateShorthand: false },
      ],
      "prefer-template": "error",
      "prefer-const": "error",
      "object-shorthand": ["error", "always", { avoidQuotes: true }],
      // Quote style (align with Prettier: singleQuote=false => double quotes)
      quotes: ["error", "double", { avoidEscape: true, allowTemplateLiterals: true }],
      "jsx-quotes": ["error", "prefer-double"],
      "quote-props": ["error", "as-needed"],
      "no-console": ["error", { allow: ["warn", "error"] }],
      "no-alert": "error",
      "default-case": "error",
      "dot-notation": "error",
      "consistent-return": "error",
      "arrow-body-style": ["error", "as-needed"],
      "prefer-destructuring": [
        "error",
        { array: false, object: true },
        { enforceForRenamedProperties: false },
      ],
      // TypeScript-specific style
      "@typescript-eslint/consistent-type-definitions": ["error", "type"],
      "@typescript-eslint/consistent-type-imports": [
        "error",
        { prefer: "type-imports", fixStyle: "inline-type-imports" },
      ],
      "@typescript-eslint/array-type": ["error", { default: "array-simple" }],
      // React style
      "react/jsx-no-useless-fragment": "error",
      "react/self-closing-comp": "error",
      "react/jsx-boolean-value": ["error", "never"],
      "react/button-has-type": "error",
      "react/no-array-index-key": "error",
      // Hooks
      "react-hooks/rules-of-hooks": "error",
      "react-hooks/exhaustive-deps": "error",
    },
  }),
  // Relax rules for declaration files (after base config so it overrides)
  {
    files: ["**/*.d.ts"],
    rules: {
      "@typescript-eslint/consistent-type-definitions": "off",
    },
  },
  // Turn off rules that might conflict with Prettier formatting
  prettier,
];
