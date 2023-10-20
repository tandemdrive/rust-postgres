//! An internal crate for `tokio-postgres`.

#![recursion_limit = "256"]

use proc_macro::TokenStream;

use syn::{parse_macro_input, DeriveInput};

mod from_row;

#[proc_macro_derive(FromRow, attributes(from_row))]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    match from_row::derive_from_row(&derive_input) {
        Ok(result) => result,
        Err(err) => err.write_errors().into(),
    }
}
