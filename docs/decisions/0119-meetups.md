# 0119 Meetups: events with no single date

Status: accepted (owner, 2026-11-24: "I like name meetup its good. Yes do a reset"); amends 0057, 0059

## Context
Pickup basketball, run clubs, pub nights and game evenings work like events (a proposal, a
plan, sessions on the calendar, roles, ratings) but have no one date. They were events with
"No set date", which confused people choosing Event, and events lost their date step to it.
The owner asked for a separate name; the Edit, Plan and Meetup mockups (§C) proposed Meetup.

## Decision
- A meetup is an event (`mode: "event"`) with subtype `meetup` (`sp_event::SUBTYPES`).
- An event plan's schedule gains `weekly`: the first usual day as `start_date` (its weekday is
  the usual day), an optional usual time, no end. `any-day` stays for "whenever members set a
  session". A dated event plan is `date` or `range`, and the event wizards offer only those.
- A proposal becomes a meetup through a **Meetup plan**: an event plan whose schedule is
  `weekly` or `any-day`. Governance reads that plan's subtype as `meetup` (coordinator
  `plans::chosen_subtype`), and the phase move that cites it sets the space's subtype, as
  production and operations plans set theirs. A private meetup is created with subtype
  `meetup` directly (Create › Private meetup).
- Each session is an activity, as for events. Feed and map cards show the next session
  (ADR "next activity", #299); a meetup's own map pin needs no date (`entity_type: "meetup"`),
  and the feed and map filters gain Meetups. Region orders a meetup by its next session.
- Its own kind in the UI: a ball glyph, teal `--kind-meetup`, the label "Meetup".

Project DNA integrity change (`sp_event`): **project network reset**. The directory and
conversation DNAs keep their hashes, but the seed drops every app of the older hApp, so the
demo content and founders are re-run afterwards.

## Consequences
"Event" now always means a set date or dates. An existing undated event (none after the reset)
would read as an event without a date. A weekly meetup's usual day is a hint: nothing is
scheduled until a member adds a session.
