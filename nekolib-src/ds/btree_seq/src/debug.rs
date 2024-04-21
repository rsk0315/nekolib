use crate::{ImmutNodeRef, InternalNode, NodeRef};

pub fn visualize<T: std::fmt::Debug>(node: ImmutNodeRef<'_, T>) {
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
    struct State {
        path: Vec<Kind>,
    }
    impl State {
        fn display<T: std::fmt::Debug>(&self, elt: &T, treelen: Option<usize>) {
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
            eprint!("{prefix}{elt:?}");
            if let Some(treelen) = treelen {
                eprint!(" ({treelen})");
            }
            eprintln!();
        }
        fn push(&mut self, k: Kind) { self.path.push(k); }
        fn pop(&mut self) { self.path.pop(); }
    }

    fn dfs<T: std::fmt::Debug>(
        node_ref: ImmutNodeRef<'_, T>,
        state: &mut State,
    ) {
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

    let mut state = State { path: vec![] };
    dfs(node, &mut state);
}

pub fn assert_invariants<T>(node: ImmutNodeRef<'_, T>) {
    fn dfs<T>(node_ref: ImmutNodeRef<'_, T>) {
        // root or non-underfull

        // `.treelen` consistency

        // `.parent` consistency
    }

    dfs(node);
}
