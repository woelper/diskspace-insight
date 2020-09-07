# diskspace-insight
Rust library to gather info about what files use up space


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
