# 0072. "Plan became leading": lead changes replayed from the order plans closed

Status: **accepted** (2026-11-03). Follows [0039](./0039-project-plans.md) (plans and their
leader), [0046](./0046-notifications-derived-on-device.md) (notifications derived on device)
and [0061](./0061-private-events-and-organizers.md) (organizer-run events). Parity item
"Plan became leading" (`docs/plans/PARITY.md`).

## Context
web-backend sends `prj-plan-lead` (projects) or `evt-plan-lead` (events) to the new leader's
author when a plan vote changes the leader of its kind (`projects_plans.py`,
`events_plans.py`: "Plan became leading", "A vote passed and your [event] plan is now
leading.", the voter as actor, linking to the page). Its Settings switch "Plan became
leading" (category `plan_leading`) is **off by default** (`DEFAULT_NOTIFICATION_CATEGORIES`).
web keeps no history of leaders: it compares the leader before and after one vote. The app
derives notifications on each device, and its plan board only says who leads now.

## Decision
- **Coordinator only** (governance zome, no network reset): `plan_lead_changes()` replays the
  approved plans in the order they closed (`plans::approved`, the same list the phase gate
  uses) and, for each close, compares its kind's leader (`plans::leader`, so organizer-run
  events' "newest approved plan leads" too) before and after. Every close that hands the lead
  to a different plan is one `LeadChange { plan, author, kind, at, by }`: `at` is that close,
  `by` the Outcome's author (the vote or organizer that closed it, web's actor).
- A plan's yes/no in the app are frozen when it closes, so leaders change only at closes;
  ratings count as they are now (the app keeps no past averages). A rating that later
  reorders the leaders sends nothing, as on web (only plan votes notify there).
- **UI:** `domain/planLeadNotifications.ts` turns the changes whose plan you wrote into web's
  card (no link target beyond the page). Settings gains web's switch, off by default:
  categories in `OFF_BY_DEFAULT` count as off until switched on (`sp_notifications_on`), so
  existing per-device choices (`sp_notifications_off`) keep their meaning. While the switch is
  off the call isn't made, so most devices pay nothing for it.

## Consequences
- Because ratings are today's, a later rating can rewrite the replayed history (a tie on
  approval decided differently): a card can appear or go away afterwards. Rare (it needs equal
  approval ratios) and harmless; keeping past averages would need an integrity change.
- One more zome call per opened project per refresh, only for people who switched it on.
- A change that happened before a device first looked shows as read (ADR 0046's read marks).
- Older gateways answer "no such function" until the coordinator update reaches the cell
  (ADR 0063); the card list treats that as no changes.
