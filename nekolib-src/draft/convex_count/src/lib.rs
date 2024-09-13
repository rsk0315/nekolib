#![allow(dead_code)]

// https://259-momone.hatenablog.com/entry/2021/07/25/025655

fn f(n: u64) -> u64 { (1..=n).map(|i| n / i).sum() }

#[test]
fn test() {
    for i in 1..=50 {
        if f(i) - f(i - 1) == 2 {
            eprintln!("{i} -> {} (+{})", f(i), f(i) - f(i - 1));
        }
    }
}
