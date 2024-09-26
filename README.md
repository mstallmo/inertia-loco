[![Crates.io](https://img.shields.io/crates/v/inertia-loco.svg)](https://crates.io/crates/inertia-loco)
[![Documentation](https://docs.rs/inertia-loco/badge.svg)](https://docs.rs/inertia-loco/)

inertia-loco
============

This crate is forked from the great [axum-inertia](https://github.com/mjhoy/axum-inertia) crate.

Implementation of the [inertia.js] protocol for loco.

Provides an `Inertia` axum extractor to render responses like so:

```rust
async fn get_posts(i: Inertia) -> impl IntoResponse {
    i.render("Posts/Index", json!({ "posts": vec!["post one", "post two"] }))
}
```

See [crate documentation] for more information.

[inertia.js]: https://inertiajs.com
[crate documentation]: https://docs.rs/inertia-loco/latest/loco_inertia/

## Making a new release

1. Spin off a `bump-vX.X.X` branch
2. Update the `CHANGELOG`; start a new `[Unreleased]` section
3. Bump the version number in `Cargo.toml`
4. Run `cargo release --execute`
5. Merge PR if all goes well

## License

This work is dual-licensed under Apache 2.0 and GPL 2.0 (or any later version).
You can choose between one of them if you use this work.

`SPDX-License-Identifier: Apache-2.0 OR MIT`
