# 0028. Hosted apps are versioned by hApp; network resets move people over

Status: accepted (2026-10-18)

## Context
Any integrity change (including the founder list in `dnas/directory/dna.yaml`) gives a new DNA
hash, which is a new network. Hosted apps already installed on the seed node keep their old DNA, so
people would silently stay on the old network.

## Decision
The gateway names each hosted app `person-<agent>-<happ8>`, where `happ8` is the first 8 hex of the
hApp bundle's sha256. At start it removes hosted apps of other versions and warms up the new one
(compiles the zomes) in the background. A device whose app is gone gets "sign in again" on resume;
signing in with the recovery phrase installs the person's app on the new network. Same key, empty profile.

## Consequences
Before launch, resets are cheap: people sign in with their words and re-enter their profile. After
launch, integrity changes must instead use genesis roll-up (docs/DEPLOYMENT.md#versions), never a silent reset.
