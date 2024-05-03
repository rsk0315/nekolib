set -u
setopt nullglob

which gmktemp >&/dev/null && mktemp=gmktemp || mktemp=mktemp

summary="$1"


cargo_test() {
    json=$1

    for dir in nekolib-src/*/; do
        local dir="${${(s:/:)dir}[2]}"
        for toml in nekolib-src/$dir/*/Cargo.toml; do
            local crate="${${(s:/:)toml}[3]}"

            local test_name=(
                $(cargo test --release --manifest-path=$toml -- -Z unstable-options --format=json)
            )
            for t in ${test_name[@]}; do
                local event
                if cargo test --release --manifest-path=$toml --exact "$test_name"; then
                    event=ok
                else
                    event=failed
                fi
                jq -n --arg test_name "$test_name" --arg event "$event" >>$json \
                   '{"name": $test_name, "type": "release", "event": $event}'
            done

            local miri_test_name
            if RUSTFLAGS=-Dunsafe_code cargo build --release; then
                # If it has no unsafety, we do not have to test against Miri.
                miri_test_name=()
            else
                miri_test_name=(
                    $(cargo miri test --manifest-path=$toml -- -Z unstable-options --format=json)
                )                
            fi
            for t in ${test_name[@]}; do
                local event
                export MIRIFLAGS=
                if cargo miri test --manifest-path=$toml --exact "$test_name"; then
                    event=ok
                else
                    event=failed
                fi
                jq -n --arg test_name "$test_name" --arg event "$event" >>$json \
                   '{"name": $test_name, "type": "stacked-borrows", "event": $event"}'

                MIRIFLAGS=-Zmiri-tree-borrows
                if cargo miri test --manifest-path=$toml --exact "$test_name"; then
                    event=ok
                else
                    event=failed
                fi
                jq -n --arg test_name "$test_name" --arg event "$event" >>$json \
                   '{"name": $test_name, "type": "stacked-borrows", "event": $event"}'
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
