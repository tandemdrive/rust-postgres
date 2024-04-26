use crate::to_statement::private::Sealed;
use crate::Statement;

pub(crate) use private::StatementType;

pub(crate) mod private {
    use crate::{Client, Error, Statement};

    pub trait Sealed {}

    pub enum StatementType<'a> {
        Statement(&'a Statement),
        Query(&'a str),
    }

    impl<'a> StatementType<'a> {
        pub async fn into_statement(self, client: &Client) -> Result<Statement, Error> {
            match self {
                StatementType::Statement(s) => Ok(s.clone()),
                StatementType::Query(s) => client.prepare(s).await,
            }
        }
    }
}

/// A trait abstracting over prepared and unprepared statements.
///
/// Many methods are generic over this bound, so that they support both a raw query string as well as a statement which
/// was prepared previously.
///
/// This trait is "sealed" and cannot be implemented by anything outside this crate.
pub trait ToStatement: Sealed {
    #[doc(hidden)]
    fn __convert(&self) -> StatementType<'_>;
}

impl ToStatement for Statement {
    fn __convert(&self) -> StatementType<'_> {
        StatementType::Statement(self)
    }
}

impl Sealed for Statement {}

impl ToStatement for str {
    fn __convert(&self) -> StatementType<'_> {
        StatementType::Query(self)
    }
}

impl Sealed for str {}

impl ToStatement for String {
    fn __convert(&self) -> StatementType<'_> {
        StatementType::Query(self)
    }
}

impl Sealed for String {}
