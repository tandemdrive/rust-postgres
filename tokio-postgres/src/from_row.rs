//! Enables mapping from [`Row`] to to an user-defined type.

use crate::{Error, Row};

#[cfg(feature = "derive")]
pub use tokio_postgres_derive::FromRow;

/// A trait for types that can be created from a Postgres row.
pub trait FromRow: Sized {
    /// Tries to perform the conversion.
    ///
    /// Will return an error if the row does not contain the expected column names.
    fn from_row(row: &Row) -> Result<Self, Error>;
}

macro_rules! tuple_impl {
    ($($T:ident[$idx:literal]),*) => {
        impl<$($T: for<'a> postgres_types::FromSql<'a>),*> FromRow for ($($T,)*) {
            fn from_row(row: &Row) -> Result<Self, Error> {
                Ok(($(row.try_get::<_, $T>($idx)?,)*))
            }
        }
    };
}

tuple_impl!(T0[0]);
tuple_impl!(T0[0], T1[1]);
tuple_impl!(T0[0], T1[1], T2[2]);
tuple_impl!(T0[0], T1[1], T2[2], T3[3]);
tuple_impl!(T0[0], T1[1], T2[2], T3[3], T4[4]);
tuple_impl!(T0[0], T1[1], T2[2], T3[3], T4[4], T5[5]);
tuple_impl!(T0[0], T1[1], T2[2], T3[3], T4[4], T5[5], T6[6]);
tuple_impl!(T0[0], T1[1], T2[2], T3[3], T4[4], T5[5], T6[6], T7[7]);
tuple_impl!(T0[0], T1[1], T2[2], T3[3], T4[4], T5[5], T6[6], T7[7], T8[8]);
tuple_impl!(T0[0], T1[1], T2[2], T3[3], T4[4], T5[5], T6[6], T7[7], T8[8], T9[9]);
