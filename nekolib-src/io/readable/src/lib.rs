use std::io::BufRead;

use input::{Readable, Source};

enum Usize1 {}
enum Isize1 {}
enum Chars {}
enum Bytes {}

impl Readable for Usize1 {
    type Output = usize;
    fn read<R: BufRead, S: Source<R>>(source: &mut S) -> usize {
        usize::read(source).checked_sub(1).unwrap()
    }
}

impl Readable for Isize1 {
    type Output = isize;
    fn read<R: BufRead, S: Source<R>>(source: &mut S) -> isize {
        isize::read(source).checked_sub(1).unwrap()
    }
}

impl Readable for Chars {
    type Output = Vec<char>;
    fn read<R: BufRead, S: Source<R>>(source: &mut S) -> Vec<char> {
        source.next_token_unwrap().chars().collect()
    }
}

impl Readable for Bytes {
    type Output = Vec<u8>;
    fn read<R: BufRead, S: Source<R>>(source: &mut S) -> Vec<u8> {
        source.next_token_unwrap().bytes().collect()
    }
}

#[test]
fn sanity_check() {
    use input::{scan, AutoSource};

    let src = AutoSource::from("10 20 3 chars bytes");
    scan! {
        from src,
        (l, r): (Usize1, usize),
        y: Isize1,
        c: Chars,
        b: Bytes,
    }

    assert_eq!(l..r, 9..20);
    assert_eq!(y, 2);
    assert!("chars".chars().eq(c));
    assert!("bytes".bytes().eq(b));
}
