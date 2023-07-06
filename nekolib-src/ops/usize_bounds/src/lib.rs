use std::ops::{
    Bound::{Excluded, Included, Unbounded},
    Range, RangeBounds,
};

#[derive(Debug)]
pub enum UsizeOob {
    StartIndexLen(usize, usize, usize),
    EndIndexLen(usize, usize, usize),
    IndexOrder(usize, usize, usize),
}

impl ToString for UsizeOob {
    fn to_string(&self) -> String {
        match self {
            UsizeOob::StartIndexLen(start, _, len) => {
                format!(
                    "range start index {start} out of range for slice of length {len}"
                )
            }
            UsizeOob::EndIndexLen(_, end, len) => {
                format!(
                    "range end index {end} out of range for slice of length {len}"
                )
            }
            UsizeOob::IndexOrder(start, end, _) => {
                format!("slice index starts at {start} but ends at {end}")
            }
        }
    }
}

impl UsizeOob {
    pub fn resolve_bounds(&self) -> Range<usize> {
        let (&start, &end, &len) = match self {
            UsizeOob::StartIndexLen(start, end, len)
            | UsizeOob::EndIndexLen(start, end, len)
            | UsizeOob::IndexOrder(start, end, len) => (start, end, len),
        };
        let end = end.min(len);
        let start = start.min(end);
        start..end
    }
}

pub trait UsizeBounds {
    fn to_range(&self, len: usize) -> Range<usize>;
    fn checked_to_range(&self, len: usize) -> Result<Range<usize>, UsizeOob>;
}

impl<R: RangeBounds<usize>> UsizeBounds for R {
    fn to_range(&self, len: usize) -> Range<usize> {
        match self.checked_to_range(len) {
            Ok(o) => o,
            Err(e) => panic!("{}", e.to_string()),
        }
    }

    fn checked_to_range(&self, len: usize) -> Result<Range<usize>, UsizeOob> {
        let start = match self.start_bound() {
            Included(&s) => s,
            Excluded(&s) => s + 1,
            Unbounded => 0,
        };
        let end = match self.end_bound() {
            Included(&e) => e + 1,
            Excluded(&e) => e,
            Unbounded => len,
        };

        if start > len {
            Err(UsizeOob::StartIndexLen(start, end, len))
        } else if end > len {
            Err(UsizeOob::EndIndexLen(start, end, len))
        } else if start > end {
            Err(UsizeOob::IndexOrder(start, end, len))
        } else {
            // start <= end <= len
            Ok(start..end)
        }
    }
}
