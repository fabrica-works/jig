# Making a project after the first start

Draft, 2026-10-23. A deployment owner's request that makes a project
while the engine runs. Not contract: once its open choices (section 9)
are answered, it is folded into the documents that own each part
(section 10), and this draft goes.

## 1. In one page

- **Today a project is only seeded,** at a deployment's first start,
  from configuration (`../protocol/configuration.md`, sections 3 and 8;
  `../domain/core.md`, 6.1). An application that offers "a new project"
  after that has no way to make one.
- **`MakeProject`** is a jig request in the deployment's scope
  (`../client/domain/contract.md`, 7.5): keyed like every request,
  admitted on the party's role in the deployment, which must be owner,
  and naming the new project's name and the configured policy it starts
  from. It is answered done with the project's number.
- **The name is checked as a seed's is** (`../domain/core.md`, 6.1): in
  shape, and held by no other project under simple case folding, seeded
  and made alike. It is required: a made project is never given
  "Project N".
- **The maker is its owner,** and its only party: deployment ownership
  gives no role in a project, and the maker adds others as an owner
  does, by roles, adoption or services.
- **Its policy is a copy of a configured one.** Configuration gains
  `policies`, numbered and never renumbered, each a whole policy as a
  seed's; the request names one, and the project keeps its own copy,
  which later configuration never changes and its owners change as any
  policy (`../domain/people.md`, 5.2).
- **Made whole, in one decision:** its number, the next the deployment
  gives and never given again; its name, policy and roster; its first
  period; and the answer. A crash keeps all of it or none.
- **The store keeps it as it keeps a seeded project,** whole: a restart
  restores made and seeded projects from the store alone, and
  configuration's projects stay ignored after the first start.
- **Bounded by the projects' capacities** (`../protocol/configuration.md`,
  section 6): seeded and made projects count alike, and a full
  deployment refuses with *limit*.

## 2. Who may ask

- **A party whose role in the deployment is owner** (`../domain/people.md`,
  section 3): the deployment's roster, project 0's, which the first
  start gives every first owner. It is the role the deployment's scope
  is admitted on (`../client/domain/contract.md`, 7.5), so making a
  project is judged as any deployment-scoped request is. The first
  owners' identities, which decide the deployment's LLM accounts
  (`../domain/authority.md`, 8.4), are not consulted: a party the
  roster makes an owner later may make projects too.
- **A service** with the deployment's owner role may ask, as a service
  makes every request a person makes (`../domain/people.md`, section 3).
- **A task never does:** making a project is running the deployment,
  which only parties do (`../domain/people.md`, 5.1, "What only parties
  do").
- **An application's request may ask it on the party's behalf,** as
  adoption does (`../client/domain/contract.md`, 3.3): its connector
  asks the core for the operation within the same decision, judged
  against the party's deployment role as `MakeProject` is. So a set-up
  form that makes a project and adopts into it is one request (section
  8).

## 3. The request and its answer

`MakeProject` names:

| Field | What it is |
|---|---|
| name | the new project's name (section 4) |
| policy | the number of a configured policy (section 6) |

It names no project: its scope is the deployment. It is not one of a
compound request's operations, which are one project's
(`../domain/people.md`, 5.1).

- **Keyed like every request** (`../client/domain/contract.md`, 3.1): a
  retry under its key is answered with the same number; another ask
  under that key is a *key conflict*. Its answer is retained for the
  answers' retention, as every ordinary request's is.
- **Done:** the project's number. The maker's `me` changes, gaining the
  project with its name, their role, owner, and their pool
  (`../client/domain/contract.md`, 5.1 and 7.2); no other party's does.
- **Refused,** with jig's kinds (`../client/domain/contract.md`,
  section 4):

| Refusal | When |
|---|---|
| *role* | the party's role in the deployment is not owner, or they have none |
| *not well formed* | the name out of shape, saying which rule: empty, past 64 bytes, a control character, white space at an end, decimal digits alone |
| *name taken* (new) | another project holds the name under simple case folding; it names no project, since the maker may not be one of its parties |
| *unknown* | no configured policy has that number |
| *limit* | the deployment's projects: authority's or people's capacity is full |
| *busy*, *not ready*, *signed out*, *version not served* | as for every request |

- **The order of checks:** sign-in and key first, as every request's;
  then the role; then the name's shape; then the policy's number; then the name's uniqueness; then the
  capacity. The first that fails is the answer.
- **Name taken** is a new refusal kind, which an owner's rename
  (`../domain/core.md`, 6.1) answers too.

## 4. The name

- **The rules are `../domain/core.md`'s, 6.1:** UTF-8 of 1 to 64 bytes,
  no control character, no white space at either end, never decimal
  digits alone; unique in the deployment under Unicode's simple case
  folding. The engine checks them at admission, against every project's
  name as committed, seeded names and given ones ("Project N")
  included.
- **Required.** A made project has the name its maker chose; "Project N"
  is a seed's fallback only. A client that wants a default fills one in,
  as temper's form does from a repository's name (section 8).
- **Two makes racing for one name:** the first commit takes it; the
  second is refused *name taken*, as two parties deciding one thing are
  (`../domain/people.md`, 5.1).
- **It rests on projects' names being built** (`../STATUS.md`, section
  3, "A project's name"): the name in configuration, in the store beside
  the policy, and in `me`. That part is this design's first piece
  (section 10).

## 5. Ownership and roles

- **The maker is the project's owner** (`../domain/people.md`, section
  4: "The party who made the project is its owner"), and its roster
  holds them alone. Their role is the configured policy's owner role,
  its authority, allotment and the proposals it decides as the policy
  defines them.
- **Other deployment owners get no role** in it. Deployment ownership
  is a role in the deployment's scope, never a bypass into a project's
  (`../client/domain/contract.md`, 7.5); a project seeded at the first
  start makes every first owner its owner because configuration names
  them, not because they own the deployment. An owner who wants others
  in it gives them roles (`SetRoles`), adopts a resource whose connector
  seeds roles (`../domain/people.md`, section 4), or makes services.
- **The maker's pool** opens with the project's first period (section
  7): the allotment the owner role gives.
- **Someone to escalate to.** The maker becomes a root's requester only
  where their role decides escalations or the policy names an escalation
  role (`../domain/people.md`, 5.1). A configured policy in which the
  owner could create no root is refused at startup (section 6), so a
  made project's owner can always start work.

## 6. Where the policy comes from

- **`policies` in jig's part of configuration** (`../protocol/configuration.md`,
  section 3): each numbered from 1 and never renumbered, as templates
  are (3.1 there); each a whole policy, in the seed's format: its
  ceiling, its spend per period, its roles, its escalation role, its
  requirements, its projections' authority, its permissions' roles for
  adoption, and its templates' overrides. An application ships its own
  in the document its set-up writes; none means no project can be made,
  and `MakeProject` answers *unknown*.
- **Checked at every start,** as a seed's policy is, so that none is
  ever found wanting when a project is made: within the
  deployment's rules, its roles' numbers and names well formed, its
  owner role able to be a root's requester (section 5), its overrides'
  templates configured. A configured policy that fails refuses the
  start, at its path.
- **Copied, never referenced.** A made project's policy is the
  configured one as it stood when the project was made. A later change
  to the configured policy, or its removal, changes no project made
  from it, as a template's change leaves the tasks made from it
  (`../protocol/configuration.md`, 3.1). Its owners change it as any
  policy (`../domain/people.md`, 5.2).
- **The rules still bound it:** a narrowing of the deployment's rules at
  a later start reaches it as it reaches every project
  (`../domain/authority.md`, section 6, effective authority).
- **Resources come by adoption:** a configured policy names patterns, not
  a project's resources, which it has none of yet; its ceiling and
  projections' grants are written for the resources a new project will
  adopt (temper's: every repository of its forges), and narrowed by an
  owner once they are known (`../domain/core.md`, 6.1).

The alternatives, and why not (section 9 asks):

| Source | For | Against |
|---|---|---|
| a configured policy, named (recommended) | the operator, or the application's set-up, decides what a new project may do, in one reviewed place; a client sends a number, not a policy; seeds' format reused | a new configuration member |
| the deployment's one default policy | simplest: no number in the request | one shape of project per deployment; a second, later, is this row's change to the first |
| a copy of an existing project's policy | no configuration | the copy carries its source's resource-specific grants and requirements; needs a project to copy, its maker one of its owners; the policy's meaning drifts as its source changes |
| the request itself, a whole policy | most flexible | every client composes and validates a policy; a deployment owner's client becomes the policy's author, with no reviewed default; the largest request jig has |

## 7. Made in one decision

The core makes the project in the decision that admits the request:

- **its number:** the deployment's next project number, kept in the
  deployment's numbers (`../protocol/store.md`, 4.4, core's row), past
  every project seeded or made, and never given again, as a task's
  number is not. The first start sets it past the highest seeded;
- **its name and its policy,** the people child's project row and the
  authority child's policy, as the first start seeds them;
- **its roster,** the maker as owner;
- **its first period,** opened with the policy's spend per period as its
  budget and no next start, as the smallest period source opens a
  seeded project's (`../protocol/configuration.md`, section 5). The
  application's root hears that the project was made, and a period
  source that keeps a calendar says its periods from then on, as for
  every project;
- **the answer,** under its key.

They are one commit: a crash keeps the project and its answer, or
neither, and a retry under the key then makes it once.

## 8. What the store, a restart and an upgrade see

- **The store.** A made project is kept as a seeded one is
  (`../protocol/store.md`, 4.4): people's family keeps its name and
  whole policy, its ceiling and escalation role among them, as
  `../domain/authority.md`, section 6 has every project's policy in the
  store; people's keeps its roster; core's deployment row keeps the next
  project number. Nothing records whether a project was seeded or made.
- **A restart** restores every project from the store alone, its policy
  never needing a configured one, as configuration's projects are
  ignored after the first start (`../protocol/configuration.md`, section
  8). Configured `policies` matter only to the next `MakeProject`.
- **The capacities** are the most projects the deployment holds, seeded
  and made: a deployment that will make projects configures room past
  its seeds. A restart whose capacities are now below the projects the
  store holds is `../protocol/configuration.md`'s, section 9 (removed
  parts at restart), as a live set past lowered limits is.
- **An upgrade.** The people family's project row and core's deployment
  row change, and the client's wire gains a request, an answer and a
  refusal: new versions of people's and core's families
  (`../protocol/store.md`, 5.1) and of the client's protocol
  (`../protocol/client.md`, section 9). Before 0.1.0 a store of the
  former versions refuses to open, saying which (`../protocol/store.md`,
  5.3), and the deployment starts again; a client of the former version
  is answered *version not served* for this request alone.

## 9. Open choices

1. **Where the policy comes from** (section 6). Options: a configured
   policy, named; the deployment's one default; a copy of an existing
   project's; the request's own. Recommended: a configured policy,
   named.
2. **Who besides the maker** (section 5). Options: the maker alone; the
   maker and every deployment owner, as a seed does. Recommended: the
   maker alone.
3. **Whether the name is required** (section 4). Options: required; or
   optional, given "Project N" past a number not yet known to the
   client. Recommended: required.

## 10. Where it goes, and in what order

Folded, once answered, into:

- `../domain/core.md`, 6.1: a project made at runtime, beside seeded.
- `../domain/people.md`, 4 and 5.1: the request's row; 5.2.
- `../domain/authority.md`, 8.4: the check on the deployment's role.
- `../client/domain/contract.md`, 3.2, 4 and 7.5: `MakeProject`, its
  answer, *name taken*.
- `../protocol/client.md`, section 3: the record.
- `../protocol/configuration.md`, sections 3, 6 and 8: `policies`; the
  capacities.
- `../protocol/store.md`, 4.4 and 5.1: the rows and versions.
- `../STATUS.md`: a row of section 3, "A project made at runtime",
  after "A project's name".

Built in two pieces, each a landing: projects' names (STATUS.md's plan,
with *name taken* for the seed's check at the store and `me`'s names),
then `MakeProject` on them.

## 11. temper's side

Not temper's design; what jig offers it.

- **The set-up's "new project".** temper's configuration, as `temper
  init` writes it, gains one `policies` entry, the policy its box's
  project 1 has today (temper's `protocol/setup.md`, the box's
  project), so a made project lands as Default does.
- **From the client:** the form's name, the repository's suggested and
  made unique against the names in `me`, is sent in `MakeProject`, then
  `Adopt` into the number it answers, each under its own key. A *name
  taken* between the two, from another client, is the form's to show.
  A failed `Adopt` leaves an empty project its maker may adopt into
  again.
- **From the server, in one decision:** temper's `SetUp` (or "New
  repository"), a deployment-scoped application request, asks the core
  to make the project and to adopt into it on the party's behalf (section
  2), so that the person never sees a project without its repository.
  This is "a follow-up act on `SetUp`'s admission" of temper's
  `protocol/setup.md`, section 15.
- **Until then** temper adopts into Default, as o/forgejo-setup's
  milestone does; nothing of that changes when `MakeProject` comes.
