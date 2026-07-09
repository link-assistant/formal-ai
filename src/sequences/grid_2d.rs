//! Two-dimensional (grid / image) pattern inference.
//!
//! ARC-style tasks and images are 2D, but the issue asks us to reuse the same
//! sequence machinery. A [`Grid`] therefore does two things: it applies the
//! spatial transforms pattern inference needs (rotation, reflection, transpose),
//! and it *projects* itself into one-dimensional sequences — rows, columns,
//! diagonals, and the boundary — so the 1D detectors in
//! [`super::patterns_1d`] and the deduplicator in [`super::compression`] can be
//! run over a grid without new machinery.
//!
//! Cells are [`LinkAddress`] values, so a grid of colours, symbols, or link ids
//! is handled uniformly. Transforms are pure and return fresh grids.

use super::store::LinkAddress;

/// A rectangular grid of link-addressed cells stored in row-major order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    rows: usize,
    cols: usize,
    cells: Vec<LinkAddress>,
}

/// A spatial transform of a grid (the dihedral group of the rectangle plus the
/// two diagonal flips available on any grid).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridTransform {
    /// Leave the grid unchanged.
    Identity,
    /// Rotate 90 degrees clockwise.
    RotateCw,
    /// Rotate 180 degrees.
    Rotate180,
    /// Rotate 90 degrees counter-clockwise.
    RotateCcw,
    /// Mirror left-to-right (reverse each row).
    ReflectHorizontal,
    /// Mirror top-to-bottom (reverse row order).
    ReflectVertical,
    /// Reflect across the main diagonal (transpose).
    Transpose,
    /// Reflect across the anti-diagonal.
    AntiTranspose,
}

/// The symmetries a grid exhibits (each is `true` when the corresponding
/// transform leaves the grid unchanged).
///
/// Each field names an independent geometric symmetry, so a flat record of
/// booleans is the clearest representation here.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GridSymmetry {
    /// Invariant under left-right mirroring.
    pub horizontal: bool,
    /// Invariant under top-bottom mirroring.
    pub vertical: bool,
    /// Invariant under 180-degree rotation.
    pub rotational_180: bool,
    /// Invariant under transposition (main-diagonal reflection); square only.
    pub diagonal: bool,
    /// Invariant under anti-diagonal reflection; square only.
    pub anti_diagonal: bool,
}

impl GridSymmetry {
    /// Whether the grid has any symmetry at all.
    #[must_use]
    pub const fn any(self) -> bool {
        self.horizontal
            || self.vertical
            || self.rotational_180
            || self.diagonal
            || self.anti_diagonal
    }
}

impl Grid {
    /// Build a grid from row-major `cells`, returning [`None`] when the length
    /// does not match `rows * cols`.
    #[must_use]
    pub fn new(rows: usize, cols: usize, cells: Vec<LinkAddress>) -> Option<Self> {
        (cells.len() == rows.checked_mul(cols)?).then_some(Self { rows, cols, cells })
    }

    /// Build a grid from a list of equal-length rows.
    #[must_use]
    pub fn from_rows(rows: &[Vec<LinkAddress>]) -> Option<Self> {
        let row_count = rows.len();
        let col_count = rows.first().map_or(0, Vec::len);
        if rows.iter().any(|row| row.len() != col_count) {
            return None;
        }
        let cells = rows.iter().flatten().copied().collect();
        Self::new(row_count, col_count, cells)
    }

    /// The number of rows.
    #[must_use]
    pub const fn rows(&self) -> usize {
        self.rows
    }

    /// The number of columns.
    #[must_use]
    pub const fn cols(&self) -> usize {
        self.cols
    }

    /// Whether the grid is square.
    #[must_use]
    pub const fn is_square(&self) -> bool {
        self.rows == self.cols
    }

    /// The cell at `(row, col)`, if in bounds.
    #[must_use]
    pub fn get(&self, row: usize, col: usize) -> Option<LinkAddress> {
        (row < self.rows && col < self.cols).then(|| self.cells[row * self.cols + col])
    }

    /// The rows as separate vectors, top to bottom.
    #[must_use]
    pub fn row_vectors(&self) -> Vec<Vec<LinkAddress>> {
        self.cells
            .chunks(self.cols.max(1))
            .map(<[_]>::to_vec)
            .collect()
    }

    /// The columns as separate vectors, left to right.
    #[must_use]
    pub fn column_vectors(&self) -> Vec<Vec<LinkAddress>> {
        (0..self.cols)
            .map(|col| {
                (0..self.rows)
                    .filter_map(|row| self.get(row, col))
                    .collect()
            })
            .collect()
    }

    /// Flatten the grid into a single row-major sequence.
    #[must_use]
    pub fn row_major(&self) -> Vec<LinkAddress> {
        self.cells.clone()
    }

    /// Flatten the grid into a single column-major sequence.
    #[must_use]
    pub fn column_major(&self) -> Vec<LinkAddress> {
        self.column_vectors().into_iter().flatten().collect()
    }

    /// The clockwise boundary cells starting from the top-left corner. Useful
    /// for detecting rotational/reflective symmetry of a frame as a 1D cycle.
    #[must_use]
    pub fn boundary(&self) -> Vec<LinkAddress> {
        if self.rows == 0 || self.cols == 0 {
            return Vec::new();
        }
        if self.rows == 1 {
            return self.cells.clone();
        }
        if self.cols == 1 {
            return self.cells.clone();
        }
        let mut boundary = Vec::new();
        let (last_row, last_col) = (self.rows - 1, self.cols - 1);
        for col in 0..self.cols {
            boundary.push(self.cells[col]);
        }
        for row in 1..self.rows {
            boundary.push(self.cells[row * self.cols + last_col]);
        }
        for col in (0..last_col).rev() {
            boundary.push(self.cells[last_row * self.cols + col]);
        }
        for row in (1..last_row).rev() {
            boundary.push(self.cells[row * self.cols]);
        }
        boundary
    }

    /// The main diagonal (top-left to bottom-right) as far as it extends.
    #[must_use]
    pub fn main_diagonal(&self) -> Vec<LinkAddress> {
        (0..self.rows.min(self.cols))
            .filter_map(|index| self.get(index, index))
            .collect()
    }

    /// The anti-diagonal (top-right to bottom-left) as far as it extends.
    #[must_use]
    pub fn anti_diagonal(&self) -> Vec<LinkAddress> {
        (0..self.rows.min(self.cols))
            .filter_map(|index| self.get(index, self.cols - 1 - index))
            .collect()
    }

    /// Apply a spatial `transform`, returning the transformed grid.
    #[must_use]
    pub fn apply(&self, transform: GridTransform) -> Self {
        match transform {
            GridTransform::Identity => self.clone(),
            GridTransform::RotateCw => self.rotate_cw(),
            GridTransform::Rotate180 => self.rotate_180(),
            GridTransform::RotateCcw => self.rotate_ccw(),
            GridTransform::ReflectHorizontal => self.reflect_horizontal(),
            GridTransform::ReflectVertical => self.reflect_vertical(),
            GridTransform::Transpose => self.transpose(),
            GridTransform::AntiTranspose => self.anti_transpose(),
        }
    }

    /// Rotate 90 degrees clockwise (`rows` and `cols` swap).
    #[must_use]
    pub fn rotate_cw(&self) -> Self {
        let mut cells = Vec::with_capacity(self.cells.len());
        for col in 0..self.cols {
            for row in (0..self.rows).rev() {
                cells.push(self.cells[row * self.cols + col]);
            }
        }
        Self {
            rows: self.cols,
            cols: self.rows,
            cells,
        }
    }

    /// Rotate 90 degrees counter-clockwise.
    #[must_use]
    pub fn rotate_ccw(&self) -> Self {
        let mut cells = Vec::with_capacity(self.cells.len());
        for col in (0..self.cols).rev() {
            for row in 0..self.rows {
                cells.push(self.cells[row * self.cols + col]);
            }
        }
        Self {
            rows: self.cols,
            cols: self.rows,
            cells,
        }
    }

    /// Rotate 180 degrees.
    #[must_use]
    pub fn rotate_180(&self) -> Self {
        let cells = self.cells.iter().rev().copied().collect();
        Self {
            rows: self.rows,
            cols: self.cols,
            cells,
        }
    }

    /// Mirror left-to-right.
    #[must_use]
    pub fn reflect_horizontal(&self) -> Self {
        let mut cells = Vec::with_capacity(self.cells.len());
        for row in 0..self.rows {
            for col in (0..self.cols).rev() {
                cells.push(self.cells[row * self.cols + col]);
            }
        }
        Self {
            rows: self.rows,
            cols: self.cols,
            cells,
        }
    }

    /// Mirror top-to-bottom.
    #[must_use]
    pub fn reflect_vertical(&self) -> Self {
        let mut cells = Vec::with_capacity(self.cells.len());
        for row in (0..self.rows).rev() {
            for col in 0..self.cols {
                cells.push(self.cells[row * self.cols + col]);
            }
        }
        Self {
            rows: self.rows,
            cols: self.cols,
            cells,
        }
    }

    /// Reflect across the main diagonal.
    #[must_use]
    pub fn transpose(&self) -> Self {
        let mut cells = Vec::with_capacity(self.cells.len());
        for col in 0..self.cols {
            for row in 0..self.rows {
                cells.push(self.cells[row * self.cols + col]);
            }
        }
        Self {
            rows: self.cols,
            cols: self.rows,
            cells,
        }
    }

    /// Reflect across the anti-diagonal.
    #[must_use]
    pub fn anti_transpose(&self) -> Self {
        let mut cells = Vec::with_capacity(self.cells.len());
        for col in (0..self.cols).rev() {
            for row in (0..self.rows).rev() {
                cells.push(self.cells[row * self.cols + col]);
            }
        }
        Self {
            rows: self.cols,
            cols: self.rows,
            cells,
        }
    }

    /// Detect the symmetries the grid exhibits. Diagonal symmetries are only
    /// considered for square grids, where the transform preserves dimensions.
    #[must_use]
    pub fn symmetries(&self) -> GridSymmetry {
        GridSymmetry {
            horizontal: self.reflect_horizontal() == *self,
            vertical: self.reflect_vertical() == *self,
            rotational_180: self.rotate_180() == *self,
            diagonal: self.is_square() && self.transpose() == *self,
            anti_diagonal: self.is_square() && self.anti_transpose() == *self,
        }
    }

    /// Return the non-identity transforms that leave the grid invariant. This is
    /// the grid's symmetry group beyond the identity, expressed as transforms an
    /// analogy could exploit.
    #[must_use]
    pub fn invariant_transforms(&self) -> Vec<GridTransform> {
        [
            GridTransform::RotateCw,
            GridTransform::Rotate180,
            GridTransform::RotateCcw,
            GridTransform::ReflectHorizontal,
            GridTransform::ReflectVertical,
            GridTransform::Transpose,
            GridTransform::AntiTranspose,
        ]
        .into_iter()
        .filter(|&transform| self.dimension_compatible(transform) && self.apply(transform) == *self)
        .collect()
    }

    /// Whether applying `transform` yields a grid of the same dimensions (so it
    /// can be compared to the original for invariance).
    const fn dimension_compatible(&self, transform: GridTransform) -> bool {
        match transform {
            GridTransform::RotateCw
            | GridTransform::RotateCcw
            | GridTransform::Transpose
            | GridTransform::AntiTranspose => self.is_square(),
            _ => true,
        }
    }

    /// Find the transform (other than identity) that maps `self` onto `other`,
    /// if one exists. This is the core of grid analogy: "the output is the input
    /// rotated / reflected".
    #[must_use]
    pub fn transform_onto(&self, other: &Self) -> Option<GridTransform> {
        [
            GridTransform::Identity,
            GridTransform::RotateCw,
            GridTransform::Rotate180,
            GridTransform::RotateCcw,
            GridTransform::ReflectHorizontal,
            GridTransform::ReflectVertical,
            GridTransform::Transpose,
            GridTransform::AntiTranspose,
        ]
        .into_iter()
        .find(|&transform| self.apply(transform) == *other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(rows: usize, cols: usize, cells: &[LinkAddress]) -> Grid {
        Grid::new(rows, cols, cells.to_vec()).expect("valid dimensions")
    }

    #[test]
    fn rejects_mismatched_dimensions() {
        assert!(Grid::new(2, 2, vec![1, 2, 3]).is_none());
        assert!(Grid::from_rows(&[vec![1, 2], vec![3]]).is_none());
    }

    #[test]
    fn rotation_round_trips() {
        let g = grid(2, 3, &[1, 2, 3, 4, 5, 6]);
        assert_eq!(g.rotate_cw().rotate_ccw(), g);
        assert_eq!(g.rotate_180().rotate_180(), g);
        // Clockwise rotation swaps dimensions to 3x2.
        let cw = g.rotate_cw();
        assert_eq!((cw.rows(), cw.cols()), (3, 2));
        assert_eq!(cw.row_major(), vec![4, 1, 5, 2, 6, 3]);
    }

    #[test]
    fn reflections_and_transpose() {
        let g = grid(2, 2, &[1, 2, 3, 4]);
        assert_eq!(g.reflect_horizontal().row_major(), vec![2, 1, 4, 3]);
        assert_eq!(g.reflect_vertical().row_major(), vec![3, 4, 1, 2]);
        assert_eq!(g.transpose().row_major(), vec![1, 3, 2, 4]);
        assert_eq!(g.anti_transpose().row_major(), vec![4, 2, 3, 1]);
    }

    #[test]
    fn detects_symmetry() {
        // Horizontally symmetric grid: each row is a palindrome.
        let g = grid(2, 3, &[1, 2, 1, 3, 4, 3]);
        let sym = g.symmetries();
        assert!(sym.horizontal);
        assert!(!sym.vertical);
        assert!(sym.any());

        // A fully symmetric square.
        let square = grid(2, 2, &[1, 1, 1, 1]);
        let sym = square.symmetries();
        assert!(sym.horizontal && sym.vertical && sym.rotational_180 && sym.diagonal);
        assert!(square
            .invariant_transforms()
            .contains(&GridTransform::Rotate180));
    }

    #[test]
    fn projections_expose_sequences() {
        let g = grid(2, 2, &[1, 2, 3, 4]);
        assert_eq!(g.row_vectors(), vec![vec![1, 2], vec![3, 4]]);
        assert_eq!(g.column_vectors(), vec![vec![1, 3], vec![2, 4]]);
        assert_eq!(g.column_major(), vec![1, 3, 2, 4]);
        assert_eq!(g.main_diagonal(), vec![1, 4]);
        assert_eq!(g.anti_diagonal(), vec![2, 3]);
    }

    #[test]
    fn boundary_walks_clockwise() {
        let g = grid(3, 3, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(g.boundary(), vec![1, 2, 3, 6, 9, 8, 7, 4]);
    }

    #[test]
    fn transform_onto_detects_analogy() {
        let input = grid(2, 3, &[1, 2, 3, 4, 5, 6]);
        let output = input.rotate_cw();
        assert_eq!(input.transform_onto(&output), Some(GridTransform::RotateCw));
        let same = input.clone();
        assert_eq!(input.transform_onto(&same), Some(GridTransform::Identity));
    }
}
