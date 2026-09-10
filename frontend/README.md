## Scripts

`dev`, `build`, `check`, `lint` and `format` do what they say. The rest:

| Script    | Does                                                                                                           |
|-----------|----------------------------------------------------------------------------------------------------------------|
| `sync`    | Regenerates SvelteKit types and Paraglide messages. Runs automatically after `pnpm install` and before `check` |
| `ci`      | `lint`, `check`, `build`. The one command CI runs                                                              |
| `gen:api` | Regenerates `src/lib/api/schema.d.ts` from the running API's OpenAPI. Commit the result                        |

`src/lib/paraglide/` and `.svelte-kit/` are generated and git-ignored. `src/lib/api/schema.d.ts`
is generated but committed, so the frontend builds without the API running.
