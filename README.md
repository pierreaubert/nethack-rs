Welcome to NetHack!

This is a clone in rust from the original [NetHack](https://nethackwiki.com/wiki/NetHack). NetHack is a rogue game like written in C since 1984.

For people in a hurry:
```
cargo run --bin nethack --release --icons=fancy
```

# But why?

Many reasons:
- It is relatively easy to do now with AI. The codebase is large 100k+ C code without test but with global variables and some funky stuff. It tooks more time than expected to "translate" it.
- Rust allow me to target WASM an WASM may allow me to run it as a smart contract on top of Polkadot. That would be fun to build and possibly attractive since you could not cheat. If Polkadot does not make it, it should run on top of Jam.
- I wanted to build a AI driver player from scratch that would play NetHack.
- I wanted to do a 3D version of NetHack

# State

## Crates

- nh-core: the translated code.
- nh-test: FFI bindings to link Rust with C.
- nh-rng: a modified random generator such that for a given seed C and Rust give identical responses.
- nh-compare: the code to compare the C and the Rust version automatically.
- nh-tui: a terminal version as close as possible as the original (text version in a terminal)
- nh-bevy: a 3d version that will work one day
- nh-assets: a library for managing the assets
- nh-polkadot: code that will one day allow to run on Polkadot via a Rust smart contract
- nh-player: a bot using rl to make progress

Claude and Gemini have been industrious: ~300k lines of Rust.

## Scripts

- 01 mapping: extract from the code base a list of object, monster, ...
- 02 items: for each item use generative AI to get a picture of the object (quality is highly variable)
- 03 models: for each item generate a mesh and use the picture to generate a texture
- 04 sounds: generate some sounds

# Let's play

Classical version (beta)
```
cargo run --bin nethack --release --icons=fancy
```

3D version (alpha)
```
cargo run --bin nethack3d --release
```

# How to contribute?

- Send PR to Github!
