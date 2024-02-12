set -u
setopt nullglob

which gmktemp >&/dev/null && mktemp=gmktemp || mktemp=mktemp

temp=$($mktemp --suffix=.json)
pass=$($mktemp --suffix=.md)
fail=$($mktemp --suffix=.md)
last_failed_dir=''

for dir in nekolib-src/*/; do
    dir="${${(s:/:)dir}[2]}"
    echo "### \`$dir\`" >>$pass
    for toml in nekolib-src/$dir/*/Cargo.toml; do
        crate="${${(s:/:)toml}[3]}"
        echo "#### \`$crate\`" >>$pass

        cargo test "$@" --manifest-path=$toml --message-format=json \
              -- -Z unstable-options --format=json \
            | jq -rsf $(dirname $0)/ci-test.jq >$temp

        if cat $temp | jq -e '.[] | select(.event == "ok")' >/dev/null; then
            # some test passed
            cat $temp \
                | jq -r '.[] | select(.event == "ok") | .name' \
                | sort | sed 's/.*/- `&`/' >>$pass
        fi
        if cat $temp | jq -e '.[] | select(.event == "failed")' >/dev/null; then
            # some test failed
            if [[ $last_failed_dir != $dir ]]; then
                echo "### \`$dir\`" >>$fail
                last_failed_dir=$dir
            fi
            echo "#### \`$crate\`" >>$fail
            cat $temp \
                | jq '.[] | select(.event == "failed") | .name' \
                | sort | sed 's/.*/- `&`/' >>$fail
        fi
    done
done

echo '## failed'
cat $fail

echo '---'

echo '## passed'
cat $pass

[[ -z "$last_failed_dir" ]]
