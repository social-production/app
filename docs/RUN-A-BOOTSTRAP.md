# Run a bootstrap + relay server

A bootstrap server tells new nodes where the others are; its relay carries traffic between
nodes behind home routers. The desktop app tries a list of them (ADR 0091), so every copy
someone runs makes the network less dependent on Railway (docs/plans/SEEDLESS.md).

You need a machine reachable from the internet on port 443 (a small VPS, or a home machine
with the port forwarded) and a domain name pointing at it.

## With Docker and Caddy (TLS certificates are automatic)

```bash
git clone --depth 1 https://github.com/social-production/app && cd app
docker build -t sp-bootstrap deploy/bootstrap
docker run -d --restart unless-stopped --name sp-bootstrap -e PORT=8080 -p 127.0.0.1:8080:8080 sp-bootstrap
# TLS in front (the relay must be https): replace the domain with yours.
caddy reverse-proxy --from bootstrap.example.org --to localhost:8080
```

Check it: `curl https://bootstrap.example.org/health` prints `{}`.

The image is the one Railway runs (`kitsune2-bootstrap-srv --production`, bootstrap and iroh
relay on one port). It keeps no data: entries live in memory and expire after 20 minutes, so a
restart costs nothing but a few minutes of re-registration.

## Use it

- **Yourself:** desktop app → Settings → Network → paste `https://bootstrap.example.org` →
  Save and restart. The app tries it first and falls back to the built-in list.
- **From the desktop app itself:** Settings → Network → Help the network runs the same server
  inside the app on a port you forward, and announces it to every desktop (ADR 0094). A
  server here is better for a machine that stays on.
- **For everyone:** open a PR adding it to `SHIPPED` in `native/src/servers.rs` (after
  Railway's). Everyone must ship the same order (ADR 0091's consequences).

Seed nodes and gateways pick their bootstrap with the `BOOTSTRAP_URL` variable.
