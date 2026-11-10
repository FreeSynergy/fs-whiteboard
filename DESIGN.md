# fs-whiteboard – Design

This repo holds the **program core** (`fs-whiteboard-core`, stage 1 of the design, no GUI and no dependency on Blinc: the core computes what is on the board, and a component draws it) and, since Thread R4, the **app** (`fs-whiteboard-app`) and the **program binary** (`fs-whiteboard`): window shell plus the generated admin frontend (ADR 0065, 0066), nothing drawn. Contract: `whiteboard.capnp` in `fs-interfaces` v0.26.0 (draft, provisional; its header is binding). Program docs: `fs-documentation/docs/en/programs/apps/whiteboard/`.

## State

Version 0.0.1 (branch `r4-admin`, not tagged). Stage 1 core; no capnp server; window = admin frontend only.

| Area | State |
|---|---|
| Boards, pages, elements with every kind in the contract | done |
| Merge per property (`merge`, `apply_remote`), type and context guarded | done; nothing calls it yet, as there is no device-to-device sync |
| Undo per editor | done; capped at 64 MiB |
| Spatial index, order keys, reading rules | done |
| Groups, copy, align, lock, reorder | done |
| Own format, SVG export; PNG and PDF `Unsupported` | done |
| Type descriptions `Board`, `Page`, `Element` (`DescribesType`, `describe/`), every capability id of `requirements.md` in one table | done; `supported` is in the table only (see "Thread R4") |
| `Whiteboard.types` in `whiteboard.capnp` (`fs-interfaces` v0.0.9) | done on the schema side; no server serves it (no capnp server) |
| App: `Backend` per type over the core, words, About, help; binary `fs-whiteboard` (window) | done, tests against the real core |
| capnp server, Presence, sharing, libraries, templates, comments | not built |
| Tests | core: 104 plus 9 on the description; app: 17 (real stores, real core); XML checked by a real parser |

Checked by an independent reviewer over several rounds before the tag.

## Decisions (the owner may change them; all provisional)

### Objects
A board is one `StoredObject` of envelope type `Collection`, `context=whiteboard`, holding its pages. An element is one `StoredObject` of store type `WhiteboardElement`, with index fields `board`, `page`, `parent`, `kind`. IRIs `https://<instance>/whiteboard/<id>`. Elements are written with the logged `put`.

### Merging
Merge units: each member of `Element` other than the kind (`page`, `parent`, `order`, `frame`, `angle`, `locked`, `deleted`, and the text-like one for that kind, connector `source` end and `target` end each as one unit, `label`); each member of `Style`; for a page each of `name`, `order`, `deleted`. The kind and a stroke's points never change after `add`.
* Each unit has a clock `(counter, actor)`, kept in the stored content under `fs:clocks`. A Lamport counter: a local change takes one more than the highest counter seen on that object. Larger `(counter, actor)` wins. No wall time anywhere in merging.
* `merge(local, remote)` is a pure function, and `apply_remote(StoredObject)` writes the result. It must be commutative, associative and idempotent; proptest proves it, over all units, including concurrent delete and move.
* `fs-hub-client::sync_once` overwrites objects, so it must not feed this store; a later sync adapter calls `apply_remote`. Not built: the hub delivers only other programs' activities.
* The rules for reading, as in the contract's header, are pure functions over the loaded elements: a deleted group or frame hides its members; a removed page hides its elements, and if none is left, the page with the highest id stays; a cycle is broken by giving the element with the lowest id on it no parent; a connector end on a deleted element stays at its last frame.

### Order
`order` is a text key between two others, from a pure function `between(a, b)`: always strictly between, with no fixed length, base-62. Appending counts upward instead of lengthening, so keys grow with the logarithm of the number of appends. Sorted as bytes, then by id. Reorder takes keys between its neighbours and keeps the order among the moved. A property test covers repeated inserts at one place.

### Undo
Pattern Command: each edit is a list of `(element, unit, old value, new value, clock it wrote)`. Stacks live in the `BoardEditor` value, in memory: one editor is one person's session. Undo writes the old value as a new change with a new clock. **A unit is skipped if someone else changed it since**: its current clock and value are not the last ones this editor wrote for it. The history keeps that chain per unit, so two undos of one property in a row work. It is capped at 64 MiB per editor, dropping the oldest edits. An undo with nothing to do in the newest edit goes on to the next one. `undo` answers `false` when nothing was left to do. `fs-objects`'s own undo is not used: it is per object image and would overwrite others' work. Delete and `add` undo through `deleted`, so nothing is lost.

### Spatial index and reading
`rstar` over the bounding boxes after rotation, per page, in memory, in one state per board shared by the board's editors, updated by edits and by `apply_remote`. `elements(page, area, cursor)` uses it, then sorts by order; hidden ones are left out. Paged by the order key and id. Non-finite numbers and negative sizes are refused at every entry.

### Groups
Pattern Composite. Coordinates are absolute, and a group's `frame` is the union of its members at reading, so a group has no position of its own. `move` on a group moves all members, nested included, each one once even if a member is named too. `delete` marks the group; members are hidden through it, and undo brings them all back. `copy` copies members and points connectors between copied elements at the copies. `group` refuses a locked member; `setParent` and `group` refuse a local cycle.

### Remote objects
`apply_remote` reads under the board lock and validates all it receives like a local entry: limits, finite numbers, a valid `order`. It refuses: anything whose existing or incoming object is not this program's by type and context, so another program's data in the store is safe; an envelope id differing from the content id; differing identity (board, kind, stroke points, connector arrow, image); a clock above `u64::MAX / 2`, so the counter cannot run out; a board with no page. Clocks on absent units are dropped when parsing. A deleted board leaves a marker, so a late remote cannot bring it back; a remote marker can only delete a board of this program.

### One repo per store
State is cached per repo, and clones and `as_actor` share it. Two repos on one store do **not** see each other's edits and can overwrite them. There is one `WhiteboardRepo` per store; this is stated in the API docs too.

### Connectors
A connector's position comes from its resolved ends, for the index, `bounds` and export. An end on a deleted element stays at its last frame. On export, a reference to something outside the exported set becomes a free end at its last position, so every export can be imported again. `connector_ends` is an inherent method, an addition beyond the contract.

### Locks
A locked element is not moved, restyled, reparented or deleted through its group or by any other path; the operation is refused as a whole. An anchor that is not itself edited, such as `before` in `reorder`, may be locked.

### Formats
Pattern Strategy per media type. Own format `application/vnd.freesynergy.whiteboard+json`, a version field, every element and page, no clocks; import gives new ids, keeps references inside the file, refuses anything over the limits and non-finite numbers, and points to nothing it does not hold. SVG export: own writer, XML-escaped, rotation as `transform`, stroke as a path, images by their file IRI. Other media types answer `Unsupported`.

### Limits
A stroke has at most 100 000 points; an import at most 32 MiB; a name or text at most 1 MiB per element; an `order` at most 1 KiB, at every entry. Colours are CSS hex only, and an image IRI must be `https` or `http`; both are checked at every entry, because an SVG viewer would act on them. Coordinates and sizes are limited to ±1e15 at every entry, remote and import included, so arithmetic stays finite.

### Feed
`subscribe(board)` over the store's change feed, filtered by that board.

### Open points
* `reorder` behind a deleted anchor goes silently to the front.
* A board's deletion marker has no clock and no authority, and can block an id nobody has used yet. It needs authority once sync exists.
* A visible element holding a key of nearly 1 KiB blocks `add`, `copy`, `group` and reorder-to-front until it is moved.

### Not built
`sync`, `board.sync`, `presence.*` (the shared Presence interface has no owner yet), `board.share`, `library.*`, `template.*`, `comment.add`, PNG and PDF export, foreign imports, stylus input beyond storing pressure, and CRDT text. Listed unsupported where a capability exists. Rendering and hit testing belong to the component. Presence can join later without touching the core: it is passing data on its own channel, and nothing here stores or expects it.

## Thread R4: admin frontend over the core (2026-11-10)

### Pattern map

| Where | Pattern | Why |
|---|---|---|
| `describe/table.rs` `Row`, `table()` | Registry (a table of rows) | one place says which command carries which capability id of `requirements.md` and whether the core can do it; description, backend and test read only this |
| `describe/switch.rs` `emit` | Adapter | the one function that turns a row into a `CommandDescription`; the only place that changes when `types.capnp` gets `supported` |
| `describe/impls.rs` | Registry entry (trait first, `DescribesType`) | `Board`, `Page`, `Element` register themselves; the standard is named in `TypeDescription.standard` |
| `describe/scope.rs` `Cmd` | Builder | labels `<type>.<command>.<field>` made once, same rule everywhere |
| `fs-whiteboard-app` `backend/*` | Repository (the frontend's `Backend`) + Command | one backend per type over the shared `Core`; `list` is `find` with an empty query |
| `core.rs` `Core` | Facade + Registry | repository, runtime handle and one editor per board (so undo works for the session) |
| `convert.rs` | Adapter (functions) | numbers (text) and yes/no (choice) to the core's types, refusing with the finding the form marks |
| `words.rs` `Words` | Strategy behind `Say` | English table behind the translation keys; replaced by the translation (ADR 0053) |
| `program.rs` | Facade | open store, make core, build the window with a view per type, launch |
| `fs-whiteboard/src/main.rs` | Command | first word picks the subcommand: `app` or none = window |

### The description
* **Standards:** `Board` is a W3C ActivityStreams 2.0 `Collection` (as stored). `Page` and `Element` have no standard (`data/standards.md`: none exists for whiteboard shapes, layers and z-order), so they follow the AS2 `Object` model and the own spec of `whiteboard.capnp`; the description's `standard` says so. Not described: stroke point pressure per point as a typed value (a stroke's `points` is multi-line text `x,y[,pressure]`); the style, merge clocks and `order` are fields or hidden as the contract has them.
* **Numbers and yes/no** have no basic type in `fs-types` (v0.0.2): they travel as `text` and a `choice` (`yes`, `no`). The app parses and refuses with `problem.character` (nearest code) until a `quantity`/`boolean` basic type exists. Reported to Strategy.
* **Lists** are `find` with an empty query. A command on one record has an `id` reference to the own type; `group` and `align` take `ids` (several) and so are list commands. A `change` command is partial: a field not sent stays, an empty list clears (to the default for style fields; for `parent`, to the page itself).
* **References** are IRIs `https://<instance>/whiteboard/<id>` for all three types (`ids.rs`); the instance is `localhost` until there is a sign-in (provisional, like the actor).

### ALL commands, one table (coordinator rule, 2026-11-10)
`describe()` returns **every** command of `requirements.md`, the ones the core cannot do too, so a frontend can show them locked ("coming"). The table `describe::table()` holds each as a row: type, name, kind, capability ids, input, result, `supported`, `reason`. Test `table_has_every_requirement` fails when an id of `requirements.md` is not in the table (the ids are a copy in `tests/requirements_ids.txt`; with `FS_REQUIREMENTS=<path to requirements.md>` it reads the live file), `table_names_no_id_the_requirements_lack` the other way, and `supported_rows_are_what_the_core_reports` compares `supported` with the core's own `capabilities()`.
* **Until `types.capnp` has the field `supported`:** the flag lives in the table only; `emit` (one function) gives every row. **The switch:** when `CommandDescription` has the field, `emit` sets it from `row.supported` and the reason from `row.reason`; no other file, no frontend change. The backends already refuse an unsupported command by name (`BackendError::Unavailable("unsupported:<name>")`, only so it cannot run); the display of "coming" comes from the description. Each backend has `supported(command)` from the table; the pinned frontend commit has no such method on `Backend`, the working tree of Thread A has (`Backend::supported`): then the three methods move into the trait impl and nothing else changes.
* Rows the core cannot do (reason in the table): PNG and PDF export, sync, live editing, share, embed, presence (show, follow, point, say), reactions, comments, history and restore, templates, libraries (load, add, search, media), timer, votes, sequences, present, layout, audit, encrypt, plug-ins, measuring and recording, page rename, document embed, tables, embedded content, annotations, element search, shape recognition, and `select` (state of the frontend, ADR 0037). `board.subscribe` is a change stream of the core; its row is unsupported in the admin because the backend answers one request at a time.

### Unsupported (named, not faked)
* **No drawing at all:** the board canvas (shapes, strokes, selection, pan and zoom, hit testing, presenting) is a **future component of `fs-components`** (ADR 0066). The window shows boards, pages and elements as records (list, detail, forms, buttons); a stroke is edited as text lines. Nothing in this repo may draw.
* **No capnp server, no command line, no `serve`:** the core has none; the app calls the core in process on the local store (`$XDG_DATA_HOME/freesynergy/whiteboard/app.db`, or `--store <file>`). `Whiteboard.types` exists in the schema; nothing serves it yet, so `fs-types`' `CatalogServer` is not wired.
* **The frontend's `Backend` is synchronous** and runs the core on a runtime with `block_on`; a slow call freezes the window (named in the frontend's DESIGN).
* Core refusals (a locked element, a cycle, a limit, the last page) arrive as `Unavailable` with the English text of the core; the frontend has no "refused" code for them yet.
* An element's `board` and the `source`/`target` free-end points are shown as fields; picking the target element is a typed IRI (the frontend has no picker for `reference` yet).
* Windows: `windows-targets: "x86_64"` in `release.yml` as `fs-streaming`; nobody has built or run it on Windows from here.

### Blockers and assumptions
* `types.capnp` has no `supported` yet (Thread T). **Assumed**, not invented: see the switch above.
* `fs-admin-frontend` is a git dependency on commit `4688cee` (full hash in `Cargo.toml`) of the branch `thread-a-admin-frontend`, not on a tag; Strategy switches it to a tag later. `fs-components-app`, `-fields` and `-styles` are pinned to the same commit so the types match.
* Blinc patches as `fs-tasks` (the fast variant: `blinc-app-fix` 5180d4e, `blinc-layout-fix` cb17db8).
