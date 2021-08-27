use core::fmt;
use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

//-----------------------------------------------------------------------------
//--- Convenient definition of a 2D matrix ------------------------------------
//-----------------------------------------------------------------------------

/// A 2D matrix implementation
#[derive(Debug, Clone)]
pub struct Matrix<T> {
    /// The number of rows
    n_rows: usize,
    /// The numer of columns
    n_cols: usize,
    /// The actual data in the matrix
    data: Vec<T>,
}

impl<T> Matrix<T>
where
    T: Clone,
{
    /// Creates a new (rows x cols) matrix
    pub fn new(rows: usize, cols: usize, default: T) -> Self {
        Self {
            n_rows: rows,
            n_cols: cols,
            data: vec![default; rows * cols],
        }
    }
}
impl<T> Matrix<T> {
    #[allow(dead_code)]
    /// Returns the number of columns
    pub fn columns(&self) -> usize {
        self.n_cols
    }
    /// Returns the number of rows
    pub fn rows(&self) -> usize {
        self.n_rows
    }
    /// Returns a reference to the item at position (i, j)
    pub fn at(&self, i: usize, j: usize) -> &T {
        let pos = self.pos(i, j);
        &self.data[pos]
    }
    /// Returns a mutable reference the item at position (i, j)
    pub fn at_mut(&mut self, i: usize, j: usize) -> &mut T {
        let pos = self.pos(i, j);
        &mut self.data[pos]
    }
    /// Returns the slice of the matrix corresponding to the ith row
    pub fn row(&self, i: usize) -> &[T] {
        &self.data[i * self.n_cols..(1 + i) * self.n_cols]
    }
    #[allow(dead_code)]
    /// Returns a mutable slice of the matrix corresponding to the ith row
    pub fn row_mut(&mut self, i: usize) -> &mut [T] {
        &mut self.data[i * self.n_cols..(1 + i) * self.n_cols]
    }
    /// Returns the offset corresponding to position (i, j) in the data vector
    fn pos(&self, i: usize, j: usize) -> usize {
        debug_assert!(
            i < self.n_rows,
            "Out of matrix bounds: position ({}, {}), dimensions: {} * {}",
            i,
            j,
            self.n_rows,
            self.n_cols
        );
        debug_assert!(
            j < self.n_cols,
            "Out of matrix bounds: position ({}, {}), dimensions: {} * {}",
            i,
            j,
            self.n_rows,
            self.n_cols
        );

        i * self.n_cols + j
    }
}

impl<T> Index<(usize, usize)> for Matrix<T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        self.at(index.0, index.1)
    }
}
impl<T> IndexMut<(usize, usize)> for Matrix<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        self.at_mut(index.0, index.1)
    }
}

impl<T> From<Vec<Vec<T>>> for Matrix<T>
where
    T: Clone + Default,
{
    fn from(mut data: Vec<Vec<T>>) -> Self {
        if data.is_empty() {
            Matrix::new(0, 0, Default::default())
        } else {
            let rows = data.len();
            let cols = if data[0].is_empty() { 0 } else { data[0].len() };

            let mut mat = Matrix::new(rows, cols, Default::default());

            for (i, mut row) in data.drain(..).enumerate() {
                for (j, val) in row.drain(..).enumerate() {
                    mat[(i, j)] = val;
                }
            }
            mat
        }
    }
}

impl<T> Display for Matrix<T>
where
    T: Display,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for (i, val) in self.data.iter().enumerate() {
            if (i + 1) % self.n_cols == 0 {
                writeln!(fmt, "{} ", val)?;
            } else {
                write!(fmt, "{} ", val)?;
            }
        }
        Ok(())
    }
}
