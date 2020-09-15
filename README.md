# diskspace-insight
Rust library to gather to see what eats up your disk space

See https://github.com/woelper/birdseye for a GUI app using this.

![Rust](https://github.com/woelper/diskspace-insight/workflows/Rust/badge.svg)

```rust
let i = scan("/home/kaputnik/Downloads");
dbg!(&i.largest_types()[..10]);
```

```
(
    "zip",
    1272982211,
),
(
    "so",
    438048904,
),
(
    "AppImage",
    203292752,
),
(
    "txt",
    160211761,
),
```
