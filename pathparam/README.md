# luminal_pathparam

Route parameter parsing for hyper and luminal.

## Why

A common pattern in modern web services is to place identifiers directly into
the request path, using some kind of route mapping that knows how to find and
name these values.

luminal_pathparam takes heavy inspiration from the `url` crate's `form_urlencoded` module, parsing into an iterable value that lets the caller do whatever further processing they might desire.

This crate is likely to evolve quite a bit. Preliminary benchmarks suggest that collecting into a map isn't as fast as even a naive, hand-coded `From<Parse<'_>>` implementation. If a few more conveniences to help implement this trait for a caller's own types can be developed, it may be possible to get some maintain decent speed combined with strong, expressive typing.

## Help Wanted

I could use more use cases for this crate as well as help in how to wire it together cleanly with the luminal router and handler crates.

## TODO

* [ ] Add examples to docs
* [ ] Add examples to example crate
