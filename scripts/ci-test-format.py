import json
import sys


def aggregate(raw):
    res = {}
    for e in raw:
        dir_ = e["dir"]
        crate = e["crate"]
        type_ = e["type"]
        event = e["event"]
        if dir_ not in res:
            res[dir_] = {}
        if crate not in res[dir_]:
            res[dir_][crate] = {
                "release": {"run": 0, "ok": 0},
                "doc": {"run": 0, "ok": 0},
                "stacked-borrows": {"run": 0, "ok": 0},
                "tree-borrows": {"run": 0, "ok": 0},
            }

        if type_ == "doc":
            ok, run = map(int, event.split("/"))
            res[dir_][crate][type_]["run"] = run
            res[dir_][crate][type_]["ok"] = ok
        else:
            res[dir_][crate][type_]["run"] += 1
            if event == "ok":
                res[dir_][crate][type_]["ok"] += 1

    return res


def prettify_col(col, *, empty_ok=False):
    OK = '"color: #1a7f37"'
    FAILED = '"color: #d1242f"'
    DIMMED = '"color: #6e7781"'

    ok, run = col["ok"], col["run"]
    if run == 0 and empty_ok:
        return "-"
    else:
        numer_style = OK if ok == run > 0 else FAILED
        denom_style = DIMMED
        return f"<span style={numer_style}>{ok}</span> / **<span style={denom_style}>{run}</span>**"


def prettify(agg):
    res = [["name", "lib", "doc", "lib (S)", "lib (T)"], ["---"] * 5]
    for (dir_k, dir_v) in agg.items():
        for (crate_k, d) in dir_v.items():
            td = [
                f"**{dir_k}**/{crate_k}",
                prettify_col(d["release"]),
                prettify_col(d["doc"]),
                prettify_col(d["stacked-borrows"], empty_ok=True),
                prettify_col(d["tree-borrows"], empty_ok=True),
            ]
            res.append(td)

    return "\n".join(map(lambda l: f"| {' | '.join(l)} |", res))


def main():
    print(prettify(aggregate(json.loads(sys.stdin.read()))))


if __name__ == "__main__":
    main()
