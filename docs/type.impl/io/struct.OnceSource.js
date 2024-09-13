(function() {
    var type_impls = Object.fromEntries([["io",[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-From%3C%26str%3E-for-OnceSource%3CBufReader%3C%26%5Bu8%5D%3E%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/input/lib.rs.html#46\">source</a><a href=\"#impl-From%3C%26str%3E-for-OnceSource%3CBufReader%3C%26%5Bu8%5D%3E%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;&amp;'a <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>&gt; for <a class=\"struct\" href=\"io/struct.OnceSource.html\" title=\"struct io::OnceSource\">OnceSource</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/std/io/buffered/bufreader/struct.BufReader.html\" title=\"struct std::io::buffered::bufreader::BufReader\">BufReader</a>&lt;&amp;'a [<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u8.html\">u8</a>]&gt;&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.from\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/input/lib.rs.html#47\">source</a><a href=\"#method.from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html#tymethod.from\" class=\"fn\">from</a>(s: &amp;'a <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>) -&gt; <a class=\"struct\" href=\"io/struct.OnceSource.html\" title=\"struct io::OnceSource\">OnceSource</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/std/io/buffered/bufreader/struct.BufReader.html\" title=\"struct std::io::buffered::bufreader::BufReader\">BufReader</a>&lt;&amp;'a [<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u8.html\">u8</a>]&gt;&gt;</h4></section></summary><div class='docblock'>Converts to this type from the input type.</div></details></div></details>","From<&'a str>","io::AutoSource"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-OnceSource%3CR%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/input/lib.rs.html#29\">source</a><a href=\"#impl-OnceSource%3CR%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;R&gt; <a class=\"struct\" href=\"io/struct.OnceSource.html\" title=\"struct io::OnceSource\">OnceSource</a>&lt;R&gt;<div class=\"where\">where\n    R: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a>,</div></h3></section></summary><div class=\"impl-items\"><section id=\"method.new\" class=\"method\"><a class=\"src rightside\" href=\"src/input/lib.rs.html#30\">source</a><h4 class=\"code-header\">pub fn <a href=\"io/struct.OnceSource.html#tymethod.new\" class=\"fn\">new</a>(source: R) -&gt; <a class=\"struct\" href=\"io/struct.OnceSource.html\" title=\"struct io::OnceSource\">OnceSource</a>&lt;R&gt;</h4></section></div></details>",0,"io::AutoSource"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Source%3CR%3E-for-OnceSource%3CR%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/input/lib.rs.html#42\">source</a><a href=\"#impl-Source%3CR%3E-for-OnceSource%3CR%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;R&gt; <a class=\"trait\" href=\"io/trait.Source.html\" title=\"trait io::Source\">Source</a>&lt;R&gt; for <a class=\"struct\" href=\"io/struct.OnceSource.html\" title=\"struct io::OnceSource\">OnceSource</a>&lt;R&gt;<div class=\"where\">where\n    R: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a>,</div></h3></section></summary><div class=\"impl-items\"><section id=\"method.next_token\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/input/lib.rs.html#43\">source</a><a href=\"#method.next_token\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"io/trait.Source.html#tymethod.next_token\" class=\"fn\">next_token</a>(&amp;mut self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/string/struct.String.html\" title=\"struct alloc::string::String\">String</a>&gt;</h4></section><section id=\"method.next_token_unwrap\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/input/lib.rs.html#15\">source</a><a href=\"#method.next_token_unwrap\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"io/trait.Source.html#method.next_token_unwrap\" class=\"fn\">next_token_unwrap</a>(&amp;mut self) -&gt; <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/string/struct.String.html\" title=\"struct alloc::string::String\">String</a></h4></section></div></details>","Source<R>","io::AutoSource"]]]]);
    if (window.register_type_impls) {
        window.register_type_impls(type_impls);
    } else {
        window.pending_type_impls = type_impls;
    }
})()
//{"start":55,"fragment_lengths":[5085]}