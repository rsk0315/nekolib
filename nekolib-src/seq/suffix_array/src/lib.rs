use std::{
    cmp::Ordering::{Equal, Greater, Less},
    collections::{BTreeMap, BTreeSet},
    ops::Index,
};

const NONE: usize = 1_usize.wrapping_neg();

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuffixArray<T: Ord> {
    buf: Vec<T>,
    sa: Vec<usize>,
}

impl<T: Ord> From<Vec<T>> for SuffixArray<T> {
    fn from(buf: Vec<T>) -> Self {
        let buf_usize = hash(&buf);
        let sa = sa_is(&buf_usize);
        Self { buf, sa }
    }
}

impl From<String> for SuffixArray<char> {
    fn from(buf: String) -> Self {
        let buf: Vec<_> = buf.chars().collect();
        Self::from_chars(buf)
    }
}

impl SuffixArray<u8> {
    pub fn from_bytes(buf: Vec<u8>) -> Self {
        let buf_usize = hash_bytes(&buf);
        let sa = sa_is(&buf_usize);
        Self { buf, sa }
    }
}

impl SuffixArray<char> {
    pub fn from_chars(buf: Vec<char>) -> Self {
        let buf_usize = hash_chars(&buf);
        let sa = sa_is(&buf_usize);
        Self { buf, sa }
    }
}

impl SuffixArray<usize> {
    pub fn from_hashed(buf: Vec<usize>) -> Self {
        assert!(Self::is_hashed(&buf));
        let buf_usize: Vec<_> =
            buf.iter().map(|x| x + 1).chain(Some(0)).collect();
        let sa = sa_is(&buf_usize);
        Self { buf, sa }
    }

    fn is_hashed(buf: &[usize]) -> bool {
        let mut count = vec![0; buf.len()];
        for &x in buf {
            count[x] += 1;
        }
        (0..buf.len())
            .find(|&i| count[i] == 0)
            .map(|i| (i..count.len()).all(|i| count[i] == 0))
            .unwrap_or(true)
    }
}

fn hash<T: Ord>(buf: &[T]) -> Vec<usize> {
    let enc: BTreeMap<_, _> = {
        let seen: BTreeSet<_> = buf.iter().collect();
        seen.into_iter().zip(0..).collect()
    };
    buf.iter()
        .map(|x| enc[x] + 1)
        .chain(Some(0)) // represents '$'
        .collect()
}

fn hash_chars(buf: &[char]) -> Vec<usize> {
    let max = match buf.iter().max() {
        Some(&c) => c as usize,
        None => return vec![0], // "$"
    };
    let enc = {
        let mut enc = vec![0; max + 1];
        for &c in buf {
            enc[c as usize] = 1;
        }
        for i in 1..=max {
            enc[i] += enc[i - 1];
        }
        enc
    };
    buf.iter().map(|&x| enc[x as usize]).chain(Some(0)).collect()
}

fn hash_bytes(buf: &[u8]) -> Vec<usize> {
    let enc = {
        let mut enc = vec![0; 256];
        for &b in buf {
            enc[b as usize] = 1;
        }
        for i in 1..=255 {
            enc[i] += enc[i - 1];
        }
        enc
    };
    buf.iter().map(|&x| enc[x as usize]).chain(Some(0)).collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LsType {
    L,
    S(bool), // is leftmost S-type
}

fn count_freq(buf: &[usize]) -> Vec<usize> {
    let mut res = vec![0; buf.len()];
    for &x in buf {
        res[x] += 1;
    }
    res
}

fn inv_perm(buf: &[usize]) -> Vec<usize> {
    let mut res = vec![0; buf.len()];
    for (i, &x) in buf.iter().enumerate() {
        res[x] = i;
    }
    res
}

fn ls_classify(buf: &[usize]) -> Vec<LsType> {
    let mut res = vec![LsType::S(false); buf.len()];
    for i in (0..buf.len() - 1).rev() {
        res[i] = match buf[i].cmp(&buf[i + 1]) {
            Less => LsType::S(false),
            Equal => res[i + 1],
            Greater => LsType::L,
        };
    }
    for i in 1..buf.len() {
        if let (LsType::L, LsType::S(_)) = (res[i - 1], res[i]) {
            res[i] = LsType::S(true);
        }
    }
    res
}

fn bucket_head(count: &[usize]) -> Vec<usize> {
    let n = count.len();
    let mut head: Vec<_> =
        std::iter::once(&0).chain(&count[..n - 1]).copied().collect();
    for i in 1..n {
        head[i] += head[i - 1];
    }
    head
}

fn bucket_tail(count: &[usize]) -> Vec<usize> {
    let mut tail = count.to_vec();
    for i in 1..count.len() {
        tail[i] += tail[i - 1];
    }
    tail
}

fn induce(buf: &[usize], sa: &mut [usize], count: &[usize], ls: &[LsType]) {
    let mut head = bucket_head(count);
    for i in 0..sa.len() {
        let j = sa[i];
        if j <= buf.len() {
            if j > 0 && ls[j - 1] == LsType::L {
                sa[head[buf[j - 1]]] = j - 1;
                head[buf[j - 1]] += 1;
            }
        }
    }
    let mut tail = bucket_tail(count);
    for i in (1..count.len()).rev() {
        let j = sa[i];
        if j <= buf.len() {
            if j > 0 && ls[j - 1] != LsType::L {
                tail[buf[j - 1]] -= 1;
                sa[tail[buf[j - 1]]] = j - 1;
            }
        }
    }
}

fn reduce(buf: &[usize], lms: &[usize], ls: &[LsType]) -> Vec<usize> {
    if lms.len() <= 1 {
        return vec![0; lms.len()];
    }

    let e = |(i0, i1)| {
        if (ls[i0], ls[i1]) == (LsType::S(true), LsType::S(true)) {
            Some(true)
        } else if ls[i0] != ls[i1] || buf[i0] != buf[i1] {
            Some(false)
        } else {
            None
        }
    };

    let mut map = vec![0; buf.len()]; // map[lms[0]] = 0
    map[lms[1]] = 1;
    let mut x = 1;
    for i in 2..lms.len() {
        let eq = buf[lms[i]] == buf[lms[i - 1]]
            && (lms[i] + 1..).zip(lms[i - 1] + 1..).find_map(e).unwrap();
        if !eq {
            x += 1;
        }
        map[lms[i]] = x;
    }
    (0..buf.len())
        .filter_map(|i| match ls[i] {
            LsType::S(true) => Some(map[i]),
            _ => None,
        })
        .collect()
}

fn sa_is(buf: &[usize]) -> Vec<usize> {
    let len = buf.len();
    let count = count_freq(buf);
    if count.iter().all(|&x| x == 1) {
        return inv_perm(buf);
    }

    let ls = ls_classify(buf);
    let mut sa = vec![NONE; len];
    let mut tail = bucket_tail(&count);
    for i in (1..len).rev().filter(|&i| ls[i] == LsType::S(true)) {
        tail[buf[i]] -= 1;
        sa[tail[buf[i]]] = i;
    }

    induce(buf, &mut sa, &count, &ls);

    // lexicographic order
    let lms: Vec<_> =
        sa.into_iter().filter(|&i| ls[i] == LsType::S(true)).collect();
    let rs_sa = sa_is(&reduce(buf, &lms, &ls));

    // appearing order
    let lms: Vec<_> = (0..len).filter(|&i| ls[i] == LsType::S(true)).collect();

    let mut tail = bucket_tail(&count);
    let mut sa = vec![NONE; len];
    for i in rs_sa.into_iter().rev() {
        let j = lms[i];
        tail[buf[j]] -= 1;
        sa[tail[buf[j]]] = j;
    }
    induce(buf, &mut sa, &count, &ls);

    sa.into_iter().collect()
}

impl<T: Ord> SuffixArray<T> {
    pub fn search(&self, pat: &[T]) -> impl Iterator<Item = usize> + '_ {
        let lo = {
            let mut lt = 1_usize.wrapping_neg();
            let mut ge = self.sa.len();
            while ge.wrapping_sub(lt) > 1 {
                let mid = lt.wrapping_add(ge.wrapping_sub(lt) / 2);
                let pos = self.sa[mid];
                match self.buf[pos..].cmp(pat) {
                    Less => lt = mid,
                    _ => ge = mid,
                }
            }
            ge
        };
        if lo >= self.sa.len() {
            return self.sa[lo..lo].iter().copied();
        }
        let hi = {
            let mut le = lo.wrapping_sub(1);
            let mut gt = self.sa.len();
            while gt.wrapping_sub(le) > 1 {
                let mid = le.wrapping_add(gt.wrapping_sub(le) / 2);
                let pos = self.sa[mid];
                let len = pat.len().min(self.buf[pos..].len());
                match self.buf[pos..pos + len].cmp(pat) {
                    Greater => gt = mid,
                    _ => le = mid,
                }
            }
            gt
        };
        self.sa[lo..hi].iter().copied()
    }

    pub fn lcpa(&self) -> Vec<usize> {
        let n = self.buf.len();
        let mut rank = vec![0; n + 1];
        for i in 0..=n {
            rank[self.sa[i]] = i;
        }
        let mut h = 0;
        let mut res = vec![0; n + 1];
        for i in 0..n {
            let j = self.sa[rank[i] - 1];
            if h > 0 {
                h -= 1;
            }
            while j + h < n && i + h < n {
                if self.buf[j + h] != self.buf[i + h] {
                    break;
                }
                h += 1;
            }
            res[rank[i]] = h;
        }
        res
    }

    pub fn into_inner(self) -> Vec<usize> { self.sa }
}

impl SuffixArray<char> {
    pub fn search_str(&self, pat: &str) -> impl Iterator<Item = usize> + '_ {
        let pat: Vec<_> = pat.chars().collect();
        self.search(&pat)
    }
}

impl<T: Ord> Index<usize> for SuffixArray<T> {
    type Output = usize;
    fn index(&self, i: usize) -> &usize { &self.sa[i] }
}

#[test]
fn sanity_check() {
    let buf = b"abracadabra".to_vec();
    let sa = SuffixArray::from_bytes(buf);
    assert_eq!(sa.sa, [11, 10, 7, 0, 3, 5, 8, 1, 4, 6, 9, 2]);
}

#[test]
fn empty_text() {
    let sa = SuffixArray::from("".to_owned());
    assert!(sa.search_str("").eq(Some(0)));
    assert!(sa.search_str("_").eq(None));
}

#[test]
fn empty_pattern() {
    let sa = SuffixArray::from("empty".to_owned());
    assert!(sa.search_str("").eq([5, 0, 1, 2, 3, 4]));
}

#[test]
fn worst_case() {
    let k = 22;
    let a: Vec<_> = (1_usize..1 << k)
        .map(|i| (k - (i & i.wrapping_neg()).trailing_zeros()) as u8)
        .collect();
    let actual = SuffixArray::from_bytes(a);

    let w = 0_usize.count_zeros();
    let expected: Vec<_> =
        (0_usize..1 << k).map(|i| !i.reverse_bits() >> (w - k)).collect();

    assert_eq!(actual.sa, expected);
}
