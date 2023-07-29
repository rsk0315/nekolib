all:
	cp crates-ls.html crates-ls.html.bak
	bash crates-ls.sh
	RUSTDOCFLAGS="--html-in-header=$$PWD/katex.html --html-in-header=$$PWD/crates-ls.html" cargo d
	mv crates-ls.html.bak crates-ls.html

open:
	cp crates-ls.html crates-ls.html.bak
	bash crates-ls.sh
	RUSTDOCFLAGS="--html-in-header=$$PWD/katex.html --html-in-header=$$PWD/crates-ls.html" cargo d --open
	mv crates-ls.html.bak crates-ls.html
