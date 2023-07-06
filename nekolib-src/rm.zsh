#! /usr/bin/zsh

mod="$1"
crate="$2"

show_usage() {
    echo "Usage: rm.zsh mod [crate]"
}

if [[ -z "$mod" ]]; then
    show_usage >&2
    exit 2
fi

if [[ -n "$crate" ]] && [[ -d "$mod/$crate" ]]; then
    pushd "$mod"
    rm -rf "$crate"
    cat Cargo.toml
    vim -N -i NONE -u NONE -s <(print '/\\[dependencies\\]/\nj/\\<'"${crate}"'\\>\nddZZ') Cargo.toml &>/dev/null
    cat Cargo.toml
    vim -N -i NONE -u NONE -s <(print 'G%j/\\<'"${crate//-/_}"'\\>\nddZZ') src/lib.rs &>/dev/null
    cat src/lib.rs
    popd
fi

if [[ -d "$mod" ]] && [[ -z "$crate" ]]; then
    rm -rf "$mod"
    pushd ../nekolib-doc
    vim -N -i NONE -u NONE -s <(print '/\\[dependencies\\]/\nj/\\<'"${mod}"'\\>\nddZZ') Cargo.toml &>/dev/null
    cat Cargo.toml
    vim -N -i NONE -u NONE -s <(print 'G%j0/\\<'"${mod//-/_}"'\\>\nddZZ') src/lib.rs &>/dev/null
    cat src/lib.rs
    popd
fi
