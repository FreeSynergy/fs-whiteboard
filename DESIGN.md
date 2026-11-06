# fs-whiteboard – Design

This repo holds the **program core**, stage 1 of the design, no GUI and no dependency on Blinc: the core computes what is on the board, and a component draws it. Contract: `whiteboard.capnp` in `fs-interfaces` v0.26.0 (draft, provisional; its header is binding). Program docs: `fs-documentation/docs/en/programs/apps/whiteboard/`.

## State

Version 0.1.0, being built. The State table is filled in as parts are finished.

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
`order` is a text key between two others, from a pure function `between(a, b)`: always strictly between, with no fixed length, base-62. Sorted as bytes, then by id. Reorder takes keys between its neighbours and keeps the order among the moved. A property test covers repeated inserts at one place.

### Undo
Pattern Command: each edit is a list of `(element, unit, old value, new value, clock it wrote)`. Stacks live in the `BoardEditor` value, in memory: one editor is one person's session. Undo writes the old value as a new change with a new clock. **A unit whose current clock is not the one this edit wrote is skipped**: someone else changed it since. `undo` answers `false` when nothing was left to do. `fs-store`'s own undo is not used: it is per object image and would overwrite others' work. Delete and `add` undo through `deleted`, so nothing is lost.

### Spatial index and reading
`rstar` over the bounding boxes after rotation, per page, in memory, in one state per board shared by the board's editors, updated by edits and by `apply_remote`. `elements(page, area, cursor)` uses it, then sorts by order; hidden ones are left out. Paged by the order key and id. Non-finite numbers and negative sizes are refused at every entry.

### Groups
Pattern Composite. Coordinates are absolute, and a group's `frame` is the union of its members at reading, so a group has no position of its own. `move` on a group moves all members, nested included, each one once even if a member is named too. `delete` marks the group; members are hidden through it, and undo brings them all back. `copy` copies members and points connectors between copied elements at the copies. `group` refuses a locked member; `setParent` and `group` refuse a local cycle.

### Formats
Pattern Strategy per media type. Own format `application/vnd.freesynergy.whiteboard+json`, a version field, every element and page, no clocks; import gives new ids, keeps references inside the file, refuses anything over the limits and non-finite numbers, and points to nothing it does not hold. SVG export: own writer, XML-escaped, rotation as `transform`, stroke as a path, images by their file IRI. Other media types answer `Unsupported`.

### Limits
A stroke has at most 100 000 points; an import at most 32 MiB. Text at most 1 MiB per element.

### Feed
`subscribe(board)` over the store's change feed, filtered by that board.

### Not built
`sync`, `board.sync`, `presence.*` (the shared Presence interface has no owner yet), `board.share`, `library.*`, `template.*`, `comment.add`, PNG and PDF export, foreign imports, stylus input beyond storing pressure, and CRDT text. Listed unsupported where a capability exists. Rendering and hit testing belong to the component. Presence can join later without touching the core: it is passing data on its own channel, and nothing here stores or expects it.
