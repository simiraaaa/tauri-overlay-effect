# Security Policy

## Supported versions
- `main` branch on GitHub is the supported track.
- Security checks are performed from `package.json` scripts.

## What we check
- `npm audit` before releases
- `npm run security:check` before release prep
- `npm run check` for TypeScript/Svelte issues
- `npm run security:check:high` for every release cut

## Response
- If a vulnerability is reported, first try non-breaking fixes (`npm run security:fix`).
- For breaking fixes, evaluate impact and apply in an isolated commit.

## Priority
- We treat `high` and `critical` vulnerabilities as highest priority.
- Low severity dependencies are still reviewed before release when practical.
