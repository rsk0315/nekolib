all:
	cp crates-ls.html crates-ls.html.bak
	bash crates-ls.sh
	RUSTDOCFLAGS="--html-in-header=$$PWD/katex.html --html-in-header=$$PWD/crates-ls.html --document-private-items" cargo d
	mv crates-ls.html.bak crates-ls.html

open:
	cp crates-ls.html crates-ls.html.bak
	bash crates-ls.sh
	RUSTDOCFLAGS="--html-in-header=$$PWD/katex.html --html-in-header=$$PWD/crates-ls.html --document-private-items" cargo d --open
	mv crates-ls.html.bak crates-ls.html

# 特定のモジュールでだけ --document-private-items 相当のことをしたい。
# 両方ビルドして適宜 mv する？ きたないけど
