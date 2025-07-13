# serde_remote_converter
Helper Macro for deriving `serde::Serialize` on foreign types

Serde (kind of) supports implementing `serde::Serialize` and serder::Deserialize on foreign types that do not implement these traits by default via [remote derive](https://serde.rs/remote-derive.html).

When the foreign types contain one or more private member, explicit getters must be provided for all fields via `#[serde(getter = "<getter>")]`. This crate provides a helper macro that allows omitting these getters, and supports structs for which one or more of these private fields have no getters.

Warning: this crate uses `unsafe` `mem::transmute` to automatically synthesize a getter for private fields. Use at your own risk.

## Usage
Follow serde's remote derive guidelines. If the foreign type has some private members, place `#[remote_converter]` on top of your matching struct definition **BEFORE** `#[derive(Serialize)]`.

```Rust
#[remote_converter]
#[derive(Serialize)]
#[serde(remote = "remote_crate::RemoteType")]
struct RemoteTypeDef<T> {
    t: T,
    member2: u32,
    ...
}
```


## Limitations
`#[remote_converter]` can only be used on structs with named fields. Enums and tuple structs are not supported.
