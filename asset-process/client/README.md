# The client

`rust-client/` is the native Rust client (package `gielinor-client`) that the harness is built on: its examples
draw objects, figures, creatures, skeletal models, effects, arenas and interfaces exactly as the game client does.
`reference-sheet/` is the program that draws the reference archive. The executables the tools call are a release
download, not in git: `BUILD.md` says where to get them, how to build each one, and lists sizes and SHA-256.

Build outputs (`target/`) are not included. `vendor/` holds the third-party crates it builds against, each with
its own licence file.

**Licence.** This client is a port of the Old School RuneScape game client. It has no licence file of its own.
