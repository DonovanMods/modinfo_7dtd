# Modinfo_7DTD

Read/Write 7 Days to Die modinfo.xml files.

Supports both version 1 and 2 of the modinfo.xml format, and can be used to convert between them.

## Usage

The crate package name is `modinfo_7dtd` to distinguish it from other Modinfo formats, but the library is simply named `modinfo`.

Please see the [documentation](https://docs.rs/modinfo_7dtd) for more information.

Add this to your `Cargo.toml`:

```toml
[dependencies]
modinfo_7dtd = "0.1"
```

Then, to use it in your code:

```rust
use modinfo::Modinfo;

let modinfo = Modinfo::new();
```

**Please note that this crate is still in development, and the API may change in the future.**
