# FreeSynergy Whiteboard — think visually

**A board for ideas that several people can change at once without a referee.**

FreeSynergy Whiteboard is an endless board with pages, shapes, text, connectors and drawings. You can group things, copy, align, lock and reorder them, and undo your own steps.

What is different here: two people changing the same board are merged property by property. If you move a shape while I recolour it, both changes stay. The merge is written so that the order of arrival does not matter, and property tests check this. Your undo only undoes your own steps and skips a property someone else has since changed. A board can be exported as SVG.

## What it can do today

Latest tag: `v0.2.0` (stage 1 of the design).

- Boards, pages and every kind of element in the contract.
- Merging of changes from two sides, commutative and repeatable; concurrent delete and move are covered.
- Undo for each editor, capped at 64 MiB.
- Groups, copy, align, lock, reorder; a fast index to find what is in view.
- Export to SVG. PNG and PDF answer "not possible".
- Tests: 104, with real stores. Checked by an independent reviewer over several rounds.

## What is not there yet

- No window, no server.
- Nothing yet sends changes between devices, so the merge is not used anywhere. Live presence, sharing, libraries, templates and comments are not built.
- The interface is a draft, marked provisional.

## Part of FreeSynergy

Whiteboard belongs to the project-management suite and is part of the "together" step of the plan. FreeSynergy is a family of programs that are free for everyone, forever, and that keep your data yours. The documentation, the manifesto and the overall plan live in [fs-documentation](https://github.com/FreeSynergy/fs-documentation); the reading version is at <https://freesynergy.github.io/fs-documentation/>. Start with the [manifesto](https://freesynergy.github.io/fs-documentation/manifesto/) if you want to know why we do this.


## For developers

- Workspace crate: `fs-whiteboard-core` (core, no GUI, no dependency on Blinc; a component draws what the core computes).
- The contract is `whiteboard.capnp` in [fs-interfaces](https://github.com/FreeSynergy/fs-interfaces) (draft, provisional). Whiteboard owns it.
- Build and test: `cargo build --workspace` and `cargo test --workspace`.
- Design and decisions: [DESIGN.md](DESIGN.md).
- Licence: MIT OR Apache-2.0.
