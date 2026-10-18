# 0010. Hosted agents for PWA users, passkey login, recovery phrase

Status: accepted (2026-10-18)

## Context
Browsers can't run a conductor, and 99% of users will be on phones.

## Decision
The seed node hosts an agent per PWA user. Login is by passkey; the gateway issues app tokens. A recovery phrase is shown at signup so the key can move to any node. No passwords or emails are stored.

## Consequences
Custodial while hosted (stated openly). Portability is the mitigation. See IDENTITY.md.
