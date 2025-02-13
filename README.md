# mdict-rs

a simple web dictionary write in rust, base on mdx format dictionary file.
it's at an early stage of development, now only support mdx version 2.0 with encrypted=2 or 0

## Usage

1. Copy your dictionary file (.db or .mdx) to the `resources/dict` directory. To use a custom location, employ the `-d` flag when running the application.
   > Post-generation tip: After database creation, you may delete the original .mdx file. Use the `-g` flag to pre-generate the database file locally before deployment.
2. If your mdx file has an associated CSS file, place it in the `resources/static/` folder, or configure a custom directory using the `-s` option.
3. Run the application.

```bash
./mdict-rs
# now open your chrome, and search
# http://localhost:8181
```

## Screenshot

![screenshot](screenshot.jpg)

## 参考

MDX的解析功能和mdx文件规范参考[mdict-analysis](https://bitbucket.org/xwang/mdict-analysis/src/master/)
和文章[MDX/MDD 文件格式解析](http://einverne.github.io/post/2018/08/mdx-mdd-file-format.html)
