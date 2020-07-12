## mdict-rs
a simple web mdcit dictionary write in rust, base on warp.

it is at an early stage of development, now only support mdx version 2.0 with encrypted=2 or 0(unencrypted).

Mdx parse code is in mdx.rs file. 

### usage
```bash
# put your mdx file in resources folder and change the MDX_PATH in main.rs
# if your mdx has separate css file, put it in static/ folder
cargo run

# now open your chrome, and search
# http://localhost:3030/q?key=<sloppy>

``` 