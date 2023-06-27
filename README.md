# nekolib

えびちゃん ([rsk0315](https://atcoder.jp/rsk0315)) の競プロライブラリ。

[旧ライブラリ](https://rsk0315.github.io/library-rs/nekolib/) を置き換えることを目指す。

## 設計に関して

以下の要件を満たすことを目指している。

- ジャッジに提出する際は、必要な部分のみを簡単に取り出せる。
    - bundler としては [nekolib-bundle](https://github.com/rsk0315/nekolib-bundle) を用いる。
- ローカルで実装する際は、Cargo.toml に依存を書くことで通常の crate のように使える。
    - `cargo atcoder new` のテンプレートに記述しておくことで毎回のコストは無視できる。
- ドキュメントは、ジャンルごとに適切に分類されて記載される。
    - algo, math, utils, … など。
- ドキュメントは gh-pages を通して自動でデプロイされる。
    - <https://rsk0315.github.io/nekolib/nekolib_doc/>

### 構成

それに伴い、下記のようなディレクトリ構成になっている。

```
.
├── LICENSE
├── README.md
├── nekolib-doc
│  ├── Cargo.toml
│  └── src
│     └── lib.rs
└── nekolib-src
   ├── genre1
   │  ├── Cargo.toml
   │  ├── genre1_lib1
   │  │  ├── Cargo.toml
   │  │  └── src
   │  │     └── lib.rs
   │  └── src
   │     └── lib.rs
   ├── genre2
   │  ├── Cargo.toml
   │  ├── genre2_lib1
   │  │  ├── Cargo.toml
   │  │  └── src
   │  │     └── lib.rs
   │  ├── genre2_lib2
   │  │  ├── Cargo.toml
   │  │  └── src
   │  │     └── lib.rs
   │  └── src
   │     └── lib.rs
   └── inner
      ├── Cargo.toml
      └── src
         └── lib.rs
```

`nekolib-doc` … ドキュメントのビルド用および dependencies の指定用。

`nekolib-doc/Cargo.toml` には、ジャンルの一覧を依存として記述する。

```toml
[dependencies]
genre1 = { path = "../nekolib-src/genre1" }
genre2 = { path = "../nekolib-src/genre2" }
```

`nekolib-doc/src/lib.rs` にも、ジャンルの一覧を記述する。

```rs
#[doc(inline)]
pub use {genre1, genre2};
```

`nekolib-src` … ライブラリの各ファイルの配置用。

`nekolib-src/genre*/Cargo.toml` には、そのジャンルのライブラリの一覧を依存として記述する。内部用マクロを提供する `inner` も含む。

```toml
[dependencies]
inner = { path = "../inner" }
genre2_lib1 = { path = "genre2_lib1" }
genre2_lib2 = { path = "genre2_lib2" }
```

`nekolib-src/genre*/src/lib.rs` にも、そのジャンルのライブラリの一覧を記述する。

```rs
use inner::doc_inline_reexport;

doc_inline_reexport! {
    genre2_lib1,
    genre2_lib2,
}
```

`nekolib-src/genre*/*/Cargo.toml` には、そのライブラリが依存するライブラリを記述する。

```toml
[dependencies]
genre2_lib1 = { path = "../genre2_lib1" }
genre1_lib1 = { path = "../../genre1/genre1_lib1" }
```

`nekolib-src/genre*/*/src/lib.rs` には、そのライブラリの内容を記述する。

### 補足

各ライブラリを crate に分けることで依存を記述できるようにし、それを元にして bundler が解決する。
ジャンルごとおよびジャンル全体で crate を作り、各々が re-export することで、ドキュメントの階層構造を整えている。

## 使用に関して

使用する側の Cargo.toml に、下記のいずれかを記述する。

```toml
# ローカルを参照する場合
nekolib = { path = "/path/to/nekolib/nekolib-doc", package = "nekolib-doc" }
```

```toml
# GitHub を参照する場合
nekolib = { git = "https://github.com/rsk0315/nekolib", package = "nekolib-doc", branch = "main" }
```

bundle に関しては [nekolib-bundle/README.md](https://github.com/rsk0315/nekolib-bundle/blob/main/README.md) を見よ。
