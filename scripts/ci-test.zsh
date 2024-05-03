set -u
setopt nullglob

which gmktemp >&/dev/null && mktemp=gmktemp || mktemp=mktemp

summary="$1"

out() {
    jq -n --arg dir "$1" --arg crate "$2" --arg test_name "$3" --arg type "$4" --arg event "$5" \
       '{"dir": $dir, "crate": $crate, "test_name": $test_name, "type": $type, "event": $event}'
}

cargo_test() {
    json=$1

    for dir in nekolib-src/*/; do
        local dir="${${(s:/:)dir}[2]}"
        for toml in nekolib-src/$dir/*/Cargo.toml; do
            local crate="${${(s:/:)toml}[3]}"

            local test_name=(
                $(cargo test --lib --release --manifest-path=$toml -- -Z unstable-options --format=json)
            )
            local event
            for t in ${test_name[@]}; do
                if cargo test --release --manifest-path=$toml -- --exact "$test_name"; then
                    event=ok
                else
                    event=failed
                fi
                out "$dir" "$crate" "$test_name" release "$event" >>$json
            done

            if cargo test --doc --manifest-path=$toml; then
                event=ok
            else
                event=failed
            fi
            out "$dir" "$crate" "$test_name" doc "$event" >>$json
            
            local miri_test_name
            if RUSTFLAGS=-Dunsafe_code cargo build --release; then
                # If it has no unsafety, we do not have to test against Miri.
                miri_test_name=()
            else
                miri_test_name=(
                    $(cargo miri test --lib --manifest-path=$toml -- -Z unstable-options --format=json)
                )
            fi
            for t in ${test_name[@]}; do
                export MIRIFLAGS=
                if cargo miri test --manifest-path=$toml -- --exact "$test_name"; then
                    event=ok
                else
                    event=failed
                fi
                out "$dir" "$crate" "$test_name" stacked-borrows "$event" >>$json

                MIRIFLAGS=-Zmiri-tree-borrows
                if cargo miri test --manifest-path=$toml -- --exact "$test_name"; then
                    event=ok
                else
                    event=failed
                fi
                out "$dir" "$crate" "$test_name" tree-borrows "$event" >>$json
            done
        done
    done
}

temp=$($mktemp --suffix=.json)
cargo_test "$temp"
{
    echo '```'
    cat "$temp" | jq -s
    echo '```'
} >>"$summary"
! cat "$temp" | jq -r .event | grep failed
