# 0122: Groups have co-managers and a photo

Status: accepted (owner, 2026-11-25, Rail and Group Chat mockup §3). Conversation network reset.

## Context
ADR 0045 gave a group one manager at a time (the creator, then whoever a leaving manager names).
The owner asked for the group edit page to "change group chat picture and give others manager
ability".

## Decision
- A `Manager { agent, on, by }` entry, listed under `Managers`: a manager makes another member a
  manager (`on`) or stops someone (themselves included) being one. Nobody makes themselves a
  manager. A manager's writes cite the newest grant or `Leave` that made them one; validation
  accepts either (`rights::manages`).
- A `Picture { bytes, by }` entry, listed under `Pictures`, at most 96 KB: the app crops it with
  the shared crop screen and re-encodes it as a JPEG of at most 512 px. The newest by someone
  managing at the time counts.
- Readers replay managers with `sp_group::Roster` (its own crate, so `sp_core` and every other
  DNA keep their hash): any manager invites, renames, removes, pins, sets the photo and makes or
  stops managers; a removed or leaving manager stops managing; only when the last one leaves is
  the group handed to the successor they named (or the longest-standing member); the last
  manager can't step down.
- The chat summary gains `managers` and `picture`; `manager` stays (the longest-managing).

## Consequences
- The integrity change is a new conversation network: every direct and group chat starts
  again (the demo chats are reseeded). Accounts, projects and events are untouched.
