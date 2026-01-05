# React + TypeScript + Vite

This is the T3Chat frontend, built with React, TypeScript, Vite, Tailwind CSS, and ShadCN components.

## Authentication

The UI supports two authentication flows, both handled by the Rust backend:

-   **Local authentication (primary)** – username/email + password login against the T3Chat database.
-   **OIDC (OpenID Connect, optional)** – “Sign in with Microsoft/identity provider” when OIDC is configured.

The backend verifies local credentials using bcrypt and verifies OIDC tokens via JWKS. The login screen will only show the OIDC option if the server reports that OIDC is enabled.

## API Base URL

The frontend talks to the backend through a single base URL exposed as `import.meta.env.VITE_API_URL`. In this project, that value is set via **Vite CLI flags** rather than `.env` files.

-   **Local development (recommended)**:

    ```bash
    cd clients/web
    pnpm dev -- --api-url http://localhost:3000
    ```

    This starts the UI on `http://localhost:3010` (by default) and points all API calls at `http://localhost:3000/api`.

-   **Production build**:

    ```bash
    cd clients/web
    pnpm run build -- --api-url https://api.example.com
    ```

See [`variables.md`](../variables.md) and the root `README.md` for more details about backend environment variables and deployment.

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
