//! Enables mapping from [`Row`] to to an user-defined type.

use crate::{Error, Row};

/// A trait for types that can be created from a Postgres row.
pub trait FromRow: Sized {
    /// Performs the conversion
    ///
    /// # Panics
    ///
    /// Panics if the row does not contain the expected column names.
    fn from_row(row: &Row) -> Self;

    /// Tries to perform the conversion.
    ///
    /// Will return an error if the row does not contain the expected column names.
    fn try_from_row(row: &Row) -> Result<Self, Error>;
}
