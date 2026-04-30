## mcproto
Minecraft protocols in Rust, async, fast and easy to use.

> Tips: This is the main crate, there's only re-export and features controlling here. 

### Features

| Feature | Description | Default |
|---------|-------------|---------|
| `codec` | Enable VarInt, VarLong, encryption and compression support | No |
| `types` | Enable Minecraft protocol types (basic and compound types) | No |

### Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
mcproto = { git = "https://github.com/MCSLTeam/mcproto" }
```

### License
GPL-3

### Links
[Repository](https://github.com/MCSLTeam/mcproto)  
[Team's Github Page](https://github.com/MCSLTeam)
