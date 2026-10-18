# Code rules

The old stack grew tangled because nothing stopped it. Here the rules are checked by CI
(`scripts/check-all.sh`), so they can't drift.

## Layers (enforced by `scripts/check-boundaries.sh`)

UI, top to bottom. A layer may import only from layers below it:

| Layer | May import | Must not import |
|---|---|---|
| `ui/src/routes/` | features, components | holo, services, stores, domain |
| `ui/src/lib/features/<f>/` | services, stores, components, domain | holo |
| `ui/src/lib/stores/` | holo (connection state only), domain | features, components |
| `ui/src/lib/services/` | holo, domain | features, components, stores |
| `ui/src/lib/components/` | domain (types) | holo, services, stores |
| `ui/src/lib/holo/` | `@holochain/client` | everything else in `#lib` |
| `ui/src/lib/domain/` | nothing | everything |

- Only `holo/` imports `@holochain/client`. Services re-export what they need from `holo/index.ts`.
- Imports use `#lib/...` with explicit extensions (SvelteKit 3 convention), e.g. `#lib/holo/index.ts`.

Rust:

| Crate kind | May depend on | Must not |
|---|---|---|
| `crates/sp_*` | std, serde | hdi, hdk, holochain |
| integrity zomes | hdi, `sp_*` | hdk, coordinator crates |
| coordinator zomes | hdk, its integrity zome, `sp_*` | other DNAs' integrity zomes |
| `services/gateway`, `native` | anything | — |

## Size

- Files: ≤ 300 lines (warning at 240) for `.rs .ts .svelte .sh`. Split by responsibility.
- Functions: ≤ 60 lines (clippy `too_many_lines`, eslint `max-lines-per-function`).
- One feature = one folder. A zome = one domain concept.

## Style

- Rust: `cargo fmt`, `clippy --all-targets -D warnings` with `pedantic`. No `unwrap()` outside tests.
  `unsafe` is forbidden.
- TS: `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`. eslint + prettier must be clean.
  Svelte 5 runes only, no legacy `export let`.
- Comments explain *why* and point to the ADR or doc. Don't narrate what the code does.

## Tests

- Validation rules: unit tests in `crates/sp_*` (fast), plus a scenario test in `tests/scenarios`
  proving the zome rejects bad data.
- Every zome function used by the UI gets at least one scenario test.
- UI flows: Playwright in `tests/e2e` (from M1).

## Decisions

Any choice a future contributor might question gets an ADR in `docs/decisions/NNNN-title.md`
(Context / Decision / Consequences). Superseding an ADR means writing a new one that links back to it.
