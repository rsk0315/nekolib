set -u
setopt nullglob

which gmktemp >&/dev/null && mktemp=gmktemp || mktemp=mktemp

pass_lib=$($mktemp --suffix=.md)
pass_doc=$($mktemp --suffix=.md)
fail_lib=$($mktemp --suffix=.md)
fail_doc=$($mktemp --suffix=.md)

cargo_test() {
    temp=$($mktemp --suffix=.json)
    pass="$1"
    fail="$2"
    opt="$3"
    last_failed_dir=''

    for dir in nekolib-src/*/; do
        dir="${${(s:/:)dir}[2]}"
        echo "### \`$dir\`" >>$pass
        for toml in nekolib-src/$dir/*/Cargo.toml; do
            crate="${${(s:/:)toml}[3]}"
            echo "- \`$crate\`" >>$pass
    
            cargo test "${(s: :)opt}" --manifest-path=$toml --message-format=json \
                  -- -Z unstable-options --format=json \
                | jq -rs 'map(select(.event != "started") | select(.type == "test"))' >$temp
    
            if cat $temp | jq -e '.[] | select(.event == "ok")' >/dev/null; then
                # some test passed
                cat $temp \
                    | jq -r '.[] | select(.event == "ok") | .name' \
                    | sort | sed 's/.*/  - `&`/' >>$pass
            fi
            if cat $temp | jq -e '.[] | select(.event == "failed")' >/dev/null; then
                # some test failed
                if [[ $last_failed_dir != $dir ]]; then
                    echo "### \`$dir\`" >>$fail
                    last_failed_dir=$dir
                fi
                echo "- \`$crate\`" >>$fail
                cat $temp \
                    | jq -r '.[] | select(.event == "failed") | .name' \
                    | sort | sed 's/.*/  - `&`/' >>$fail
            fi
        done
    done
}

cargo_test "$pass_lib" "$fail_lib" '--lib --release'
(( ? == 0 )) || fail=t
cargo_test "$pass_doc" "$fail_doc" '--doc --release'
(( ? == 0 )) || fail=t

echo '## :x: failed (`--lib`)'
cat $fail_lib

echo '---'

echo '## :x: failed (`--doc`)'
cat $fail_doc

echo '---'

echo '## :sparkles: passed (`--lib`)'
cat $pass_lib

echo '---'

echo '## :sparkles: passed (`--doc`)'
cat $pass_doc

[[ $fail == t ]]
