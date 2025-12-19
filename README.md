# 项目介绍

一个 npm 包，使用 rust 实现从一个html片段中提取出可能的数据列表元素

# 注意

1. macos m 芯片下，需要使用 `--target aarch64-apple-darwin` 编译。否则会报错

```
yarn build --target aarch64-apple-darwin
```

而不是直接 yarn build

2. 不能直接 npm publish， 要走 gitlab action CI 发布； 发布时需要在 gitlab 中配置 npm token； commit 信息需要以版本包开头，如"1.0.4 publish"，否则会因为 commit msg 不匹配跳过发布
