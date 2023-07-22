pub struct DisjointSet(Vec<usize>);

impl DisjointSet {
    pub fn new(n: usize) -> Self { Self((0..n).collect()) }
    pub fn unite(&mut self, u: usize, v: usize) -> bool {
        if self.0[u] == self.0[v] {
            return false;
        }
        let n = self.0.len();
        for i in 0..n {
            if self.0[i] == self.0[u] {
                self.0[i] = self.0[v];
            }
        }
        true
    }
    pub fn equiv(&self, u: usize, v: usize) -> bool {
        self.repr(u) == self.repr(v)
    }
    pub fn repr(&self, u: usize) -> usize { self.0[u] }
    pub fn count(&self, u: usize) -> usize {
        let n = self.0.len();
        (0..n).filter(|&i| self.0[i] == self.0[u]).count()
    }
}
