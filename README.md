## mdict-rs

a simple web dictionary write in rust, base on mdx format dictionary file.
it's at an early stage of development, now only support mdx version 2.0 with encrypted=2 or 0(unencrypted).

### usage

```bash
# put your mdx file in /resources/mdx/en folder and change the MDX_PATH in main.rs
# if your mdx file has separate css file, put it in /resources/static/ folder
cargo run

# now open your chrome, and search
# http://localhost:8080

``` 

### 参考

MDX的解析功能和mdx文件规范参考[mdict-analysis](https://bitbucket.org/xwang/mdict-analysis/src/master/)
和文章[MDX/MDD 文件格式解析](http://einverne.github.io/post/2018/08/mdx-mdd-file-format.html)

### header string

```text
牛津
b'<Dictionary GeneratedByEngineVersion="2.0" RequiredEngineVersion="2.0" Format="Html" KeyCaseSensitive="No" StripKey="No" Encrypted="0" RegisterBy="EMail"  Title="\xe7\x89\x9b\xe6\xb4\xa5\xe9\xab\x98\xe9\x98\xb6\xe8\x8b\xb1\xe6\xb1\x89\xe5\x8f\x8c\xe8\xa7\xa3\xe8\xaf\x8d\xe5\x85\xb8(\xe7\xae\x80\xe4\xbd\x93) \xe7\xac\xac 8 \xe7\x89\x88" Encoding="UTF-8" CreationDate="2015-12-25" Compact="No" Compat="No" Left2Right="Yes" DataSourceFormat="107" StyleSheet=""/>\r\n'

朗文
b'<Dictionary GeneratedByEngineVersion="2.0" RequiredEngineVersion="2.0" Format="Html" KeyCaseSensitive="No" StripKey="Yes" Encrypted="2" RegisterBy="EMail"  Title="\xe6\x9c\x97\xe6\x96\x87\xe5\xbd\x93\xe4\xbb\xa3\xe5\x8f\x8c\xe8\xa7\xa3" Encoding="UTF-8" CreationDate="2015-7-18" Compact="Yes" Compat="Yes" Left2Right="Yes" DataSourceFormat="106" StyleSheet=""/>\r\n'

汉语
b'<Dictionary GeneratedByEngineVersion="2.0" RequiredEngineVersion="2.0" Format="Html" KeyCaseSensitive="No" StripKey="Yes" Encrypted="0" RegisterBy="EMail"  Title="\xe6\xb1\x89\xe8\x8b\xb1\xe8\xaf\x8d\xe5\x85\xb8\xef\xbc\x88\xe7\xac\xac\xe4\xb8\x89\xe7\x89\x88\xef\xbc\x89" Encoding="UTF-8" CreationDate="2020-5-24" Compact="Yes" Compat="Yes" Left2Right="Yes" DataSourceFormat="106" StyleSheet=""/>\r\n'
```

pub(crate)和pub有什么区别?
unwrap和?有什么区别?