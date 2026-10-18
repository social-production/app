# Static UI. VITE_SP_GATEWAYS (comma-separated gateway URLs, seed node first) is baked in
# at build time; Railway passes service variables as build args.
FROM node:22-bookworm-slim AS build
WORKDIR /src
COPY package.json package-lock.json ./
COPY ui/package.json ui/
RUN npm ci
COPY ui ui
ARG VITE_SP_GATEWAYS
RUN VITE_SP_GATEWAYS="$VITE_SP_GATEWAYS" npm run build -w ui

FROM caddy:2-alpine
COPY deploy/ui/Caddyfile /etc/caddy/Caddyfile
COPY --from=build /src/ui/build /srv
