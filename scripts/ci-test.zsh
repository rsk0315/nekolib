set -u
setopt nullglob

which gmktemp &>/dev/null && mktemp=gmktemp || mktemp=mktemp

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
                $(cargo test --lib --release --manifest-path=$toml \
                        -- -Z unstable-options --format=json --list \
                    | jq -rs 'map(select(.event == "discovered").name)[]')
            )
            echo "test_name: (${test_name[*]})" >&2
            local event
            for t in ${test_name[@]}; do
                if cargo test --lib --release --manifest-path=$toml -- --exact "$t"; then
                    event=ok
                else
                    event=failed
                fi
                out "$dir" "$crate" "$t" release "$event" >>$json
            done

            local doc=$(
                cargo test --doc --manifest-path=$toml \
                      -- -Z unstable-options --format=json \
                    | jq -rs 'map(select(.type == "test"))
                              | [(map(select(.event == "ok")) | length),
                                 (map(select([.event == "ok", .event == "failed"] | any)) | length)]
                              | join("/")')
            out "$dir" "$crate" doc doc "$doc" >>$json
            
            local miri_test_name
            if RUSTFLAGS=-Dunsafe_code cargo build --release --manifest-path=$toml &>/dev/null; then
                # If it has no unsafety, we do not have to test against Miri.
                miri_test_name=()
            else
                miri_test_name=(
                    $(cargo miri test --lib --manifest-path=$toml \
                            -- -Z unstable-options --format=json --list \
                          | jq -rs 'map(select(.event == "discovered").name)[]')
                )
            fi
            echo "miri_test_name: (${miri_test_name[*]})" >&2
            for t in ${miri_test_name[@]}; do
                export MIRIFLAGS=
                if cargo miri test --lib --manifest-path=$toml -- --exact "$t"; then
                    event=ok
                else
                    event=failed
                fi
                out "$dir" "$crate" "$t" stacked-borrows "$event" >>$json

                MIRIFLAGS=-Zmiri-tree-borrows
                if cargo miri test --lib --manifest-path=$toml -- --exact "$t"; then
                    event=ok
                else
                    event=failed
                fi
                out "$dir" "$crate" "$t" tree-borrows "$event" >>$json
            done
        done
    done
}

temp=$($mktemp --suffix=.json)
cargo_test "$temp"
{
    cat "$temp" | jq -s | python $(dirname $0)/ci-test-format.py
    cat <<EOF

---
<details>
<summary>raw JSON</summary>

\`\`\`json
$(cat $temp | jq -s)
\`\`\`
</details>
EOF
} >>"$summary"
! cat "$temp" | jq -r .event | grep -q failed
