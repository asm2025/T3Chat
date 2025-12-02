# React + TypeScript + Vite

This template provides a minimal setup to get React working in Vite with HMR and some ESLint rules.

## Authentication

This app uses OIDC (OpenID Connect) for authentication. Authentication is handled by the backend server, which verifies OIDC tokens via JWKS.

## Environment Variables

The frontend reads environment variables from `.env` files. See [`variables.md`](../variables.md) for a complete reference of all environment variables.

### Available Variables

-   `VITE_API_URL` – Backend API base URL (optional, defaults to `http://localhost:3000`)

**Note**: Only variables prefixed with `VITE_` are exposed to the frontend code.

### Environment Modes

-   Place frontend configuration in `.env.development`, `.env.staging`, and `.env.release`
-   Vite automatically loads `.env.<mode>`; the dev script passes `--mode <APP_ENV>` so the selected backend environment stays in sync
-   For manual runs use `pnpm run dev -- --mode staging` or `pnpm run build -- --mode release`

### Example Configuration

**Development (`ui/.env.development`):**
```bash
VITE_API_URL=http://localhost:3000
```

**Production (`ui/.env.release`):**
```bash
VITE_API_URL=https://api.example.com
```

📖 **For complete environment variable documentation**, see [`variables.md`](../variables.md)

## Expanding the ESLint configuration

If you are developing a production application, we recommend updating the configuration to enable type-aware lint rules:

```js
export default tseslint.config({
  extends: [
    // Remove ...tseslint.configs.recommended and replace with this
    ...tseslint.configs.recommendedTypeChecked,
    // Alternatively, use this for stricter rules
    ...tseslint.configs.strictTypeChecked,
    // Optionally, add this for stylistic rules
    ...tseslint.configs.stylisticTypeChecked,
  ],
  languageOptions: {
    // other options...
    parserOptions: {
      project: ['./tsconfig.node.json', './tsconfig.app.json'],
      tsconfigRootDir: import.meta.dirname,
    },
  },
})
```

You can also install [eslint-plugin-react-x](https://github.com/Rel1cx/eslint-react/tree/main/packages/plugins/eslint-plugin-react-x) and [eslint-plugin-react-dom](https://github.com/Rel1cx/eslint-react/tree/main/packages/plugins/eslint-plugin-react-dom) for React-specific lint rules:

```js
// eslint.config.js
import reactX from 'eslint-plugin-react-x'
import reactDom from 'eslint-plugin-react-dom'

export default tseslint.config({
  plugins: {
    // Add the react-x and react-dom plugins
    'react-x': reactX,
    'react-dom': reactDom,
  },
  rules: {
    // other rules...
    // Enable its recommended typescript rules
    ...reactX.configs['recommended-typescript'].rules,
    ...reactDom.configs.recommended.rules,
  },
})
```

Currently, two official plugins are available:

- [@vitejs/plugin-react](https://github.com/vitejs/vite-plugin-react/blob/main/packages/plugin-react) uses [Babel](https://babeljs.io/) for Fast Refresh
- [@vitejs/plugin-react-swc](https://github.com/vitejs/vite-plugin-react/blob/main/packages/plugin-react-swc) uses [SWC](https://swc.rs/) for Fast Refresh
