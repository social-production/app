# 0005. Three DNAs: directory, project (clone per project), conversation (clone per chat)

Status: accepted (2026-10-18)

## Context
Members should hold their projects' data. DMs must be private. Discovery must be global.

## Decision
directory = global (profiles, standing, project index, feeds). project = cloned per project. conversation = cloned per DM/group, with a membrane admitting participants only.

## Consequences
Sharding by membership comes for free. Cross-project queries go through the directory index. Many cells per agent, so hosted-agent scale is watched (NETWORK.md).
