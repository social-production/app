# 0112. Privacy switches as display rules on tagged self-links

Status: **accepted** (2026-11-18, UX session, carrying the parity queue; PARITY.md item 2).

## Context
web's Settings > Privacy has four switches. "Require approval to follow you" shipped with
ADR 0054. The other three are still missing:
- **Hide profile activity from non-followers:** only followers see your projects, threads,
  events and public posts on your profile.
- **Hide personal posts from non-followers:** people who don't follow you don't see your posts.
- **Hide my public activity from others' personal feeds:** your public projects, threads and
  events stay out of the follow-based Home feeds of the people who follow you.

ADR 0054 already settled what these can mean on a public network. They are **display rules,
not secrecy**: the app doesn't show the content, but it is still in the directory and project
networks. ADR 0054 also planned to store them with the approval switch.

Approval is a `RequiresFollowApproval` link from the person to themselves. Its validation
checks the author, base and target, but not the tag.

## Decision
- **Stored as tagged self-links, coordinator only, no reset.** Each switch is a
  `RequiresFollowApproval` self-link whose tag names it: `privacy:hide-profile`,
  `privacy:hide-posts` or `privacy:hide-from-feeds`. Approval stays the untagged link. The
  integrity zome already allows these links, so the directory network keeps its hash and
  nobody signs up again. A new link type would have reset the directory and wiped the live
  demo content, for no gain in what the network can enforce.
- **Reading the switches:**
  - `Policy::of` counts only the untagged link as approval, and collects the tagged ones.
  - `FollowView` gains `privacy` (the person's switches).
  - Each `FollowRow` in `following` gains the followed person's switches.
  - A new `privacy_of(agents)` returns the switches for a batch of authors. The gateway lets
    guests call it, as it does `follow_view`.
  - `set_privacy(name, on)` writes or deletes one switch and refuses unknown names.
- **Where the device applies them:**
  - **Profile:** with hide-profile on, someone who isn't an accepted follower, guests
    included, sees "@name shares their activity with followers" instead of the feed. The
    header, counts and Follow button stay. You always see your own profile.
  - **Posts:** with hide-posts on, a non-follower doesn't see that author's posts in Home's
    "Following + popular" rows, Search, or on the profile.
  - **Home:** with hide-from-feeds on, that person's projects, threads and events drop out
    of the Home feed of everyone who follows them, and out of the "commented" rows. Their
    posts still show to followers.
- **Copy:** Settings says it plainly: "Everyone on the network can still reach it; the app
  just doesn't show it."
- **Old coordinators:** a desktop app that hasn't updated would read a tagged link as
  approval, so its follows of that person would show as pending. Desktop builds update
  together with the hApp, so this lasts one update at most.

## Consequences
- Parity's Settings rows are complete apart from the feed split. The app's Home / Global /
  Region tabs already separate the personal feed from the public ones, so web's "Separate
  Public and Personal feeds" switch has nothing to switch.
- This is not real privacy. Real privacy (encrypted follower-only posts) would need keys per
  follower set. That is a later ADR if the owner wants it.
