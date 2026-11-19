# 0115: Desktop downloads from a rolling prerelease

Date: 2026-11-19. Status: accepted.

## Context

The owner asked for download links on the About page for every kind of device. The desktop
builds (ADR 0021, 0090) were only CI artifacts: they need a GitHub login and expire after
7 days. There is no native phone app yet (ADR 0020: iOS is blocked), but the web app installs
from the browser on Android and iPhone.

## Decision

- `desktop.yml` gains a `publish` job on main (pushes and nightly) that uploads each platform
  that built to one prerelease tagged `desktop-latest`, under fixed names:
  `social-production-windows.exe`, `social-production-macos.tar.gz` (Apple silicon),
  `social-production-linux.tar.gz`. A platform that failed keeps its last good file.
- The About page's "Get the app" card links to
  `releases/download/desktop-latest/<name>`, puts the visitor's own device first, and tells
  phone users how to install the web app (Chrome's Install app, Safari's Add to Home Screen).
- They are called previews and say they are unsigned, so Windows SmartScreen and macOS
  Gatekeeper warn. Signed installers (Tauri bundles) are a later step.

## Consequences

Links are stable and need no login. Each build replaces the last, so there is no version
history until real releases start.
