set -u
setopt nullglob

which gmktemp >&/dev/null && mktemp=gmktemp || mktemp=mktemp

pass_lib=$($mktemp --suffix=.md)
pass_doc=$($mktemp --suffix=.md)
fail_lib=$($mktemp --suffix=.md)
fail_doc=$($mktemp --suffix=.md)
notest_lib=$($mktemp --suffix=.md)
notest_doc=$($mktemp --suffix=.md)

cargo_test() {
    temp=$($mktemp --suffix=.json)
    pass="$1"
    fail="$2"
    notest="$3"
    opt="$4"
    last_pass_dir=''
    last_fail_dir=''
    last_notest_dir=''
    failed=''

    for dir in nekolib-src/*/; do
        dir="${${(s:/:)dir}[2]}"
        for toml in nekolib-src/$dir/*/Cargo.toml; do
            crate="${${(s:/:)toml}[3]}"
    
            cargo test "${(s: :)opt}" --manifest-path=$toml --message-format=json \
                  -- -Z unstable-options --format=json \
                | jq -rs 'map(select(.event != "started") | select(.type == "test"))' >$temp

            tested=''
            if cat $temp | jq -e '.[] | select(.event == "ok")' >/dev/null; then
                # some test passed
                tested=t
                if [[ $last_pass_dir != $dir ]]; then
                    echo "### \`$dir\`" >>$pass
                    last_pass_dir=$dir
                fi
                echo "- \`$crate\`" >>$pass
                cat $temp \
                    | jq -r '.[] | select(.event == "ok") | .name' \
                    | sort | sed 's/.*/  - `&`/' >>$pass
            fi
            if cat $temp | jq -e '.[] | select(.event == "failed")' >/dev/null; then
                # some test failed
                tested=t
                failed=t
                if [[ $last_fail_dir != $dir ]]; then
                    echo "### \`$dir\`" >>$fail
                    last_fail_dir=$dir
                fi
                echo "- \`$crate\`" >>$fail
                cat $temp \
                    | jq -r '.[] | select(.event == "failed") | .name' \
                    | sort | sed 's/.*/  - `&`/' >>$fail
            fi
            if [[ -z "$tested" ]]; then
                if [[ $last_notest_dir != $dir ]]; then
                    echo "### \`$dir\`" >>$notest
                    last_notest_dir=$dir
                fi
                echo "- \`$crate\`" >>$notest
            fi
        done
    done

    [[ "$failed" == t ]]
}

cargo_test "$pass_lib" "$fail_lib" "$notest_lib" '--lib --release'
(( ? == 0 )) || fail=t
cargo_test "$pass_doc" "$fail_doc" "$notest_doc" '--doc --release'
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

echo '---'

echo '## :smiling_face_with_tear: not tested (`--lib`)'
cat $notest_lib

echo '---'

echo '## :smiling_face_with_tear: not tested (`--doc`)'
cat $notest_doc

[[ "$failed" != t ]]
