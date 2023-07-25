use std::marker::PhantomData;
use std::ptr::NonNull;

/// ```compile_fail
/// struct Base {
///     buf: Vec<String>,
///     len: usize, // some other data
/// }
///
/// enum Entry<'a> {
///     Occupied(OccupiedEntry<'a>),
///     Vacant(VacantEntry),
/// }
/// use Entry::Occupied;
///
/// struct OccupiedEntry<'a> {
///     handle: &'a mut String,
///     base: &'a mut Base,
/// }
/// struct VacantEntry;
///
/// impl Base {
///     pub fn new() -> Self { Self { buf: vec![], len: 0 } }
///     pub fn entry(&mut self, key: usize) -> Entry {
///         match self.buf.get_mut(key) {
///             Some(handle) => Occupied(OccupiedEntry { base: self, handle }),
///             None => unimplemented!(),
///         }
///     }
/// }
/// ```
///
/// ```text
/// error[E0499]: cannot borrow `*self` as mutable more than once at a time
///   --> src/lib.rs:26:60
///    |
/// 22 |     pub fn entry(&mut self, key: usize) -> Entry {
///    |                  - let's call the lifetime of this reference `'1`
/// 23 |         match self.buf.get_mut(key) {
///    |               --------------------- first mutable borrow occurs here
/// 24 |             Some(handle) => Occupied(OccupiedEntry { base: self, handle }),
///    |                             -------------------------------^^^^-----------
///    |                             |                              |
///    |                             |                              second mutable borrow occurs here
///    |                             returning this value requires that `self.buf` is borrowed for `'1`
/// ```
///
///
/// ## References
/// - <https://www.reddit.com/r/rust/comments/11f45re>
/// - <https://doc.rust-lang.org/nomicon/lifetime-mismatch.html#improperly-reduced-borrows>
pub struct DormantMutRef<'a, T> {
    ptr: NonNull<T>,
    _marker: PhantomData<&'a mut T>,
}

unsafe impl<'a, T> Sync for DormantMutRef<'a, T> where &'a mut T: Sync {}
unsafe impl<'a, T> Send for DormantMutRef<'a, T> where &'a mut T: Send {}

impl<'a, T> DormantMutRef<'a, T> {
    pub fn new(t: &'a mut T) -> (&'a mut T, Self) {
        let ptr = NonNull::from(t);
        let new_ref = unsafe { &mut *ptr.as_ptr() };
        (new_ref, Self { ptr, _marker: PhantomData })
    }
    pub unsafe fn awaken(self) -> &'a mut T {
        unsafe { &mut *self.ptr.as_ptr() }
    }
    pub unsafe fn reborrow(&mut self) -> &'a mut T {
        unsafe { &mut *self.ptr.as_ptr() }
    }
    pub unsafe fn reborrow_shared(&self) -> &'a T {
        unsafe { &*self.ptr.as_ptr() }
    }
}

#[test]
fn sanity_check() {
    let mut x = 0;
    let y = {
        let (x, dormant_x) = DormantMutRef::new(&mut x);
        *x += 1;
        unsafe { dormant_x.awaken() }
    };
    *y += 1;
    assert_eq!(x, 2);
}

#[cfg(test)]
mod test {
    use crate::DormantMutRef;

    struct Base {
        buf: Vec<Option<String>>,
        len: usize,
    }

    enum Entry<'a> {
        Occupied(OccupiedEntry<'a>),
        Vacant(VacantEntry<'a>),
    }
    use Entry::{Occupied, Vacant};

    struct OccupiedEntry<'a> {
        key: usize,
        handle: &'a mut String,
        dormant_base: DormantMutRef<'a, Base>,
    }

    struct VacantEntry<'a> {
        key: usize,
        handle: Option<&'a mut Option<String>>,
        dormant_base: DormantMutRef<'a, Base>,
    }

    impl Base {
        pub fn new() -> Self { Self { buf: vec![], len: 0 } }
        pub fn entry(&mut self, key: usize) -> Entry {
            let (self_, dormant_self) = DormantMutRef::new(self);
            match self_.buf.get_mut(key) {
                Some(Some(v)) => Occupied(OccupiedEntry {
                    key,
                    handle: v,
                    dormant_base: dormant_self,
                }),
                Some(v) => Vacant(VacantEntry {
                    key,
                    handle: Some(v),
                    dormant_base: dormant_self,
                }),
                None => Vacant(VacantEntry {
                    key,
                    handle: None,
                    dormant_base: dormant_self,
                }),
            }
        }
    }

    impl<'a> Entry<'a> {
        pub fn and_modify<F>(self, f: F) -> Self
        where
            F: FnOnce(&mut String),
        {
            match self {
                Occupied(mut entry) => {
                    f(entry.get_mut());
                    Occupied(entry)
                }
                Vacant(entry) => Vacant(entry),
            }
        }
        pub fn key(&self) -> usize {
            match *self {
                Occupied(ref entry) => entry.key(),
                Vacant(ref entry) => entry.key(),
            }
        }
        pub fn or_default(self) -> &'a mut String {
            match self {
                Occupied(entry) => entry.into_mut(),
                Vacant(entry) => entry.insert(Default::default()),
            }
        }
        pub fn or_insert(self, default: String) -> &'a mut String {
            match self {
                Occupied(entry) => entry.into_mut(),
                Vacant(entry) => entry.insert(default),
            }
        }
        pub fn or_insert_with<F: FnOnce() -> String>(
            self,
            default: F,
        ) -> &'a mut String {
            match self {
                Occupied(entry) => entry.into_mut(),
                Vacant(entry) => entry.insert(default()),
            }
        }
        pub fn or_insert_with_key<F: FnOnce(usize) -> String>(
            self,
            default: F,
        ) -> &'a mut String {
            match self {
                Occupied(entry) => entry.into_mut(),
                Vacant(entry) => {
                    let value = default(entry.key());
                    entry.insert(value)
                }
            }
        }
    }

    impl<'a> OccupiedEntry<'a> {
        pub fn get(&self) -> &String { &*self.handle }
        pub fn get_mut(&mut self) -> &mut String { self.handle }
        pub fn insert(&mut self, value: String) -> String {
            std::mem::replace(self.handle, value)
        }
        pub fn into_mut(self) -> &'a mut String { self.handle }
        pub fn key(&self) -> usize { self.key }
        pub fn remove(self) -> String { self.remove_entry().1 }
        pub fn remove_entry(self) -> (usize, String) {
            let res = std::mem::take(self.handle);
            unsafe { self.dormant_base.awaken() }.len -= 1;
            (self.key, res)
        }
    }

    impl<'a> VacantEntry<'a> {
        pub fn key(&self) -> usize { self.key }
        pub fn insert(self, value: String) -> &'a mut String {
            match self.handle {
                None => {
                    let key = self.key;
                    let base = unsafe { self.dormant_base.awaken() };
                    base.buf.resize_with(key, || None);
                    base.buf.push(Some(value));
                    base.len += 1;
                    base.buf.last_mut().unwrap().as_mut().unwrap()
                }
                Some(handle) => {
                    let was_none = handle.is_none();
                    *handle = Some(value);
                    let base = unsafe { self.dormant_base.awaken() };
                    if was_none {
                        base.len += 1;
                    }
                    base.buf.get_mut(self.key).unwrap().as_mut().unwrap()
                }
            }
        }
    }

    #[test]
    fn entry() {
        let mut base = Base::new();

        assert_eq!(base.entry(0).key(), 0);

        base.entry(0).or_insert("zero".to_owned());
        assert_eq!(base.buf[0].as_ref().unwrap(), "zero");
        assert_eq!(base.len, 1);

        base.entry(0).or_insert_with(|| "xxx".to_owned());
        assert_eq!(base.buf[0].as_ref().unwrap(), "zero");
        assert_eq!(base.len, 1);

        base.entry(2).or_insert_with_key(|_| "two".to_owned());
        assert!(base.buf[1].is_none());
        assert_eq!(base.buf[2].as_ref().unwrap(), "two");
        assert_eq!(base.len, 2);

        base.entry(2).and_modify(|v| *v = "second".to_owned());
        assert_eq!(base.len, 2);

        if let Occupied(o) = base.entry(2) {
            assert_eq!(o.get(), "second");
            assert_eq!(o.remove(), "second");
            assert_eq!(base.len, 1);
        }

        base.entry(1).or_default();
        assert_eq!(base.len, 2);
        assert!(base.buf[1].as_ref().unwrap().is_empty());
        if let Occupied(mut o) = base.entry(1) {
            o.insert("first".to_owned());
            assert_eq!(o.get(), "first");
        }
    }
}
