#! /bin/sh

du -d2 \
    | cut -f2 \
    | awk -F/ '$3 { printf("{\"%s\": \"%s\"}\n", $2, $3) }' \
    | sort \
    | jq 'to_entries[]' \
    | jq -s 'group_by(.key) | map({(map(.key)[0]): map(.value | select(. != "src"))}) | add'
