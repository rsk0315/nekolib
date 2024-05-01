use crate::{
    ForceResult, Handle, ImmutNodeRef, InternalNode, NodeRef, MIN_BUFLEN,
};

pub fn visualize<T: std::fmt::Debug>(node: ImmutNodeRef<'_, T>) {
    visualize_with(node, |elt| format!("{elt:?}"));
}

pub fn visualize_with<T, F, D>(node: ImmutNodeRef<'_, T>, fmt: F)
where
    F: Fn(&T) -> D,
    D: AsRef<str>,
{
    #[derive(Clone, Copy, Eq, PartialEq)]
    enum Kind {
        IntPre,
        IntFst,
        IntMid,
        IntLst,
        IntSgl,
        LeafFst,
        LeafMid,
        LeafLst,
    }
    use Kind::*;
    struct State<F> {
        path: Vec<Kind>,
        fmt: F,
    }
    impl<F> State<F> {
        fn display<T, D>(&self, elt: &T, treelen: Option<usize>)
        where
            F: Fn(&T) -> D,
            D: AsRef<str>,
        {
            let mut prefix = "".to_owned();
            for i in 0..self.path.len() - 1 {
                let k0 = self.path[i];
                let k1 = self.path[i + 1];
                let term = i + 1 == self.path.len() - 1;
                prefix += match (k0, k1) {
                    (IntPre, IntPre) => "    ",
                    (IntPre, IntFst) if term => "┌── ",
                    (IntPre, IntFst | IntMid | IntLst) => "│   ",
                    (IntPre, LeafFst) => "┌── ",
                    (IntPre, LeafMid | LeafLst) => "├── ",
                    (IntFst, IntFst) if term => "├── ",
                    (IntFst, IntPre | IntFst | IntMid | IntLst) => "│   ",
                    (IntFst, LeafFst | LeafMid | LeafLst) => "├── ",
                    (IntMid, IntFst) if term => "├── ",
                    (IntMid, IntPre | IntFst | IntMid | IntLst) => "│   ",
                    (IntMid, LeafFst | LeafMid | LeafLst) => "├── ",
                    (IntLst, IntPre) => "│   ",
                    (IntLst, IntFst) if term => "└── ",
                    (IntLst, IntFst | IntMid | IntLst) => "    ",
                    (IntLst, LeafFst | LeafMid) => "├── ",
                    (IntLst, LeafLst) => "└── ",
                    (IntSgl, IntPre) => "│   ",
                    (IntSgl, IntFst) if term => "└── ",
                    (IntSgl, IntFst | IntMid | IntLst) => "    ",
                    (IntSgl, LeafFst | LeafMid) => "├── ",
                    (IntSgl, LeafLst) => "└── ",
                    (LeafFst | LeafMid | LeafLst, _) => unreachable!(),
                    (_, IntSgl) => unreachable!(),
                };
            }
            eprint!("{prefix}{}", (self.fmt)(&elt).as_ref());
            if let Some(treelen) = treelen {
                eprint!(" ({treelen})");
            }
            eprintln!();
        }
        fn push(&mut self, k: Kind) { self.path.push(k); }
        fn pop(&mut self) { self.path.pop(); }
    }

    fn dfs<T, F, D>(node_ref: ImmutNodeRef<'_, T>, state: &mut State<F>)
    where
        F: Fn(&T) -> D,
        D: AsRef<str>,
    {
        unsafe {
            let ptr = node_ref.node.as_ptr();
            let init_len = (*ptr).buflen as usize;
            let height = node_ref.height;
            if height > 0 {
                let ptr = node_ref.node.cast::<InternalNode<T>>().as_ptr();
                let child = |i: usize| {
                    NodeRef::from_node(
                        (*ptr).children[i].assume_init(),
                        height - 1,
                    )
                };

                {
                    state.push(IntPre);
                    dfs(child(0), state);
                    state.pop();
                }
                for i in 0..init_len {
                    if init_len == 1 {
                        state.push(IntSgl);
                    } else if i == 0 {
                        state.push(IntFst);
                    } else if i == init_len - 1 {
                        state.push(IntLst);
                    } else {
                        state.push(IntMid);
                    }
                    let elt = (*ptr).data.buf[i].assume_init_ref();
                    state.display(elt, Some((*ptr).treelen));
                    dfs(child(i + 1), state);
                    state.pop();
                }
            } else {
                for i in 0..init_len {
                    if i == 0 {
                        state.push(LeafFst);
                    } else if i == init_len - 1 {
                        state.push(LeafLst);
                    } else {
                        state.push(LeafMid);
                    }
                    let elt = (*ptr).buf[i].assume_init_ref();
                    state.display(elt, None);
                    state.pop();
                }
            }
        }
    }

    let mut state = State { path: vec![], fmt };
    dfs(node, &mut state);
}

pub fn assert_invariants<T>(node: ImmutNodeRef<'_, T>) {
    fn dfs<T>(node_ref: ImmutNodeRef<'_, T>) {
        let buflen = node_ref.buflen() as usize;
        let height = node_ref.height;

        // root or non-underfull
        if node_ref.parent().is_some() {
            assert!(
                buflen >= MIN_BUFLEN,
                "every non-root node must have at least {} nodes; but has only {}",
                MIN_BUFLEN,
                buflen
            );
        }

        // `.treelen` consistency
        if let ForceResult::Internal(internal) = node_ref.force() {
            let children: usize = (0..=buflen)
                .map(|i| internal.child(i as _).unwrap().treelen())
                .sum();
            let expected = buflen + children;
            assert_eq!(
                internal.treelen(),
                expected,
                "the subtree size is inconsistent"
            );
        }

        if height > 0 {
            for i in 0..=buflen {
                let child = node_ref.get_child(i).unwrap();

                // `.parent` consistency
                let Handle { node: child_par, idx, .. } =
                    child.parent().unwrap();
                assert_eq!(
                    child_par.node, node_ref.node,
                    "every child node must have a link to the correct parent node"
                );
                assert_eq!(
                    idx as usize, i,
                    "every child node must remember the correct sibling index; expected: {}, actual: {}",
                    i, idx
                );

                // recurse
                dfs(child);
            }
        }
    }

    assert!(node.parent().is_none(), "the root node must have no parent");
    assert!(
        node.buflen() >= 1,
        "the root node must have at least one children"
    );
    dfs(node);
}
