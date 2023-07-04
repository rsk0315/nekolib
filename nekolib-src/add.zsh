#! /usr/bin/zsh

mod="$1"
crate="$2"

show_usage() {
    echo "$0 mod [crate]"
}

if [[ -z "$mod" ]]; then
    show_usage >&2
    exit 2
fi

if ! [[ -d "$mod" ]]; then
    cargo new --lib "$mod"
    pushd ../nekolib-doc
    echo "$mod = { path = \"../nekolib-src/$mod\" }" >> Cargo.toml
    vim -N -i NONE -u NONE -s <(print "jf{a${mod},\x1bZZ") src/lib.rs &>/dev/null
    rustfmt src/lib.rs
    cat src/lib.rs
    popd
fi

if [[ -n "$crate" ]] && ! [[ -d "$mod/$crate" ]]; then
    pushd "$mod"
    cargo new --lib "$crate"
    echo "$crate = { path = \"$crate\" }" >> Cargo.toml
    vim -N -i NONE -u NONE -s <(print "GO    ${crate},\x1bVj%j!sortZZ") src/lib.rs &>/dev/null
    cat src/lib.rs
    popd
fi
