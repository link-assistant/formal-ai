//! Unit tests for 2D grid transforms and symmetry (issue #531). Externalised
//! from `src/sequences/grid_2d.rs`.

use formal_ai::sequences::{Grid, GridTransform, LinkAddress};

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
