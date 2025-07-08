use std::collections::HashSet;
use thiserror::Error;

type Matrix<T, const M: usize, const N: usize> = [[T; N]; M];

/// Returns a `Matrix` (`M`x`N`) that stores values of type `T` such as every element of the matrix
/// is the result of `f(line, column)`.
fn build_matrix<T, const M: usize, const N: usize, F>(f: F) -> Matrix<T, M, N>
where
    F: Fn(usize, usize) -> T,
    T: Default + Copy,
{
    let mut m = [[T::default(); N]; M];
    for i in 0..M {
        for j in 0..N {
            m[i][j] = f(i, j);
        }
    }
    m
}

/// Returns a `Matrix` (`M`x`N`) with the diagonal mapped by the function `f` which receives the
/// line of the diagonal element.
fn with_diag<T, const M: usize, const N: usize, F>(mut m: Matrix<T, M, N>, f: F) -> Matrix<T, M, N>
where
    F: Fn(usize) -> T,
{
    for i in 0..M {
        m[i][i] = f(i);
    }
    m
}

/// Indicates that is a error on the `Topology`.
#[derive(Error, Debug, Clone)]
pub enum TopologyError {
    /// Happens when there is a try to access a invalid node.
    #[error("invalid node index `{0}`")]
    InvalidNode(usize),
    /// Happens when a node tries to connect to itself.
    #[error("cannot connect node `{0}` to itself")]
    LoopConnection(usize),
}

/// A nework topology.
#[derive(Debug, Clone, Copy)]
pub struct Topology<const N: usize> {
    adjacency_matrix: [[bool; N]; N],
}

impl<const N: usize> Topology<N> {
    /// Returns a array of booleans where each element indicates if `n` is connected
    /// to the node of that index.
    ///
    /// - Errors: if `n` is a invalid node id.
    pub fn connections(&self, n: usize) -> Result<[bool; N], TopologyError> {
        if n >= N {
            Err(TopologyError::InvalidNode(n))
        } else {
            Ok(self.adjacency_matrix[n])
        }
    }

    /// Returns a array of booleans where each element indicates if `n` is connected
    /// to the node of that index.
    ///
    /// - Panics: if `n` is a invalid node id.
    pub fn connections_unchecked(&self, n: usize) -> [bool; N] {
        self.adjacency_matrix[n]
    }

    /// Returns a boolean indicating that `n1` is connected to `n2` in `Topology`.
    ///
    /// - Errors: if `n1` or `n2` are invalid node ids.
    pub fn connected(&self, n1: usize, n2: usize) -> Result<bool, TopologyError> {
        let max = usize::max(n1, n2);
        if max >= N {
            Err(TopologyError::InvalidNode(max))
        } else {
            Ok(self.adjacency_matrix[n1][n2])
        }
    }

    /// Returns a boolean indicating that `n1` is connected to `n2` in `Topology`.
    ///
    /// - Panics: if `n1` or `n2` are invalid node ids.
    pub fn connected_unchecked(&self, n1: usize, n2: usize) -> bool {
        self.adjacency_matrix[n1][n2]
    }
}

/// Classic topology types that can be used as base to build a new `Topology` using the
/// `TopologyBuilder`.
#[derive(Clone, Debug, Default)]
enum TopologyKind {
    #[default]
    Null,
    Full,
    Ring,
    Star,
}

impl TopologyKind {
    /// Returns the adjacency matrix.
    fn matrix<const N: usize>(&self) -> Matrix<bool, N, N> {
        match self {
            Self::Null => [[false; N]; N],
            Self::Full => with_diag([[true; N]; N], |_| false),
            Self::Ring => build_matrix(|i, j| {
                let is_left_neigh = if i == 0 { j == N - 1 } else { i - 1 == j };
                let is_right_neigh = ((i + 1) % N) == j;

                is_left_neigh || is_right_neigh
            }),
            Self::Star => build_matrix(|i, j| i == N / 2 && i != j),
        }
    }
}

/// Builder type to construct and personalize a `Topology`.
#[derive(Clone, Debug, Default)]
pub struct TopologyBuilder {
    base: TopologyKind,
    deltas: HashSet<(usize, usize)>,
}

impl TopologyBuilder {
    /// Creates a new `TopologyBuilder` with no connections.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `TopologyBuilder` of an fully connected network where all nodes are connected
    /// (execept that a node can't connect to itself).
    pub fn full() -> Self {
        Self {
            base: TopologyKind::Full,
            deltas: HashSet::new(),
        }
    }

    /// Creates a new `TopologyBuilder` of an ring network.
    pub fn ring() -> Self {
        Self {
            base: TopologyKind::Ring,
            deltas: HashSet::new(),
        }
    }

    /// Creates a new `TopologyBuilder` of an star network.
    pub fn star() -> Self {
        Self {
            base: TopologyKind::Star,
            deltas: HashSet::new(),
        }
    }

    /// Connects `n1` to `n2` and `n2` to `n1` since network connections usually are two-sided.
    pub fn connect(mut self, n1: usize, n2: usize) -> Self {
        if n1 < n2 {
            self.deltas.insert((n1, n2));
        } else {
            self.deltas.insert((n2, n1));
        }
        self
    }

    /// Disconnects `n1` to `n2` and `n2` to `n1` since network connections usually are two-sided.
    pub fn disconnect(mut self, n1: usize, n2: usize) -> Self {
        if n1 < n2 {
            self.deltas.remove(&(n1, n2));
        } else {
            self.deltas.remove(&(n2, n1));
        }
        self
    }

    /// Build a new `Topology` with `N` nodes.
    ///
    /// Can error if:
    /// 1) There is some invalid node index;
    /// 2) There is some a reflexive connection (a node can't connect to itself);
    pub fn build<const N: usize>(self) -> Result<Topology<N>, TopologyError> {
        let mut m = self.base.matrix();
        for (i, j) in self.deltas.into_iter() {
            let max = usize::max(i, j);
            if max >= N {
                return Err(TopologyError::InvalidNode(max));
            }
            if i == j {
                return Err(TopologyError::LoopConnection(i));
            }

            m[i][j] = true;
            m[j][i] = true;
        }

        Ok(Topology {
            adjacency_matrix: m,
        })
    }
}
