//! Errors.

use fallible_iterator::FallibleIterator;
use postgres_protocol::message::backend::{ErrorFields, ErrorResponseBody};
use std::error::Error as StdError;
use std::fmt;
use std::io;

pub use self::sqlstate::*;

#[allow(clippy::unreadable_literal)]
mod sqlstate;

/// The severity of a Postgres error or notice.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Severity {
    /// PANIC
    Panic,
    /// FATAL
    Fatal,
    /// ERROR
    Error,
    /// WARNING
    Warning,
    /// NOTICE
    Notice,
    /// DEBUG
    Debug,
    /// INFO
    Info,
    /// LOG
    Log,
}

impl fmt::Display for Severity {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match *self {
            Severity::Panic => "PANIC",
            Severity::Fatal => "FATAL",
            Severity::Error => "ERROR",
            Severity::Warning => "WARNING",
            Severity::Notice => "NOTICE",
            Severity::Debug => "DEBUG",
            Severity::Info => "INFO",
            Severity::Log => "LOG",
        };
        fmt.write_str(s)
    }
}

impl Severity {
    fn from_str(s: &str) -> Option<Severity> {
        match s {
            "PANIC" => Some(Severity::Panic),
            "FATAL" => Some(Severity::Fatal),
            "ERROR" => Some(Severity::Error),
            "WARNING" => Some(Severity::Warning),
            "NOTICE" => Some(Severity::Notice),
            "DEBUG" => Some(Severity::Debug),
            "INFO" => Some(Severity::Info),
            "LOG" => Some(Severity::Log),
            _ => None,
        }
    }
}

/// A Postgres error or notice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbError {
    severity: Box<str>,
    parsed_severity: Option<Severity>,
    code: SqlState,
    message: Box<str>,
    detail: Option<Box<str>>,
    hint: Option<Box<str>>,
    position: Option<ErrorPosition>,
    where_: Option<Box<str>>,
    schema: Option<Box<str>>,
    table: Option<Box<str>>,
    column: Option<Box<str>>,
    datatype: Option<Box<str>>,
    constraint: Option<Box<str>>,
    file: Option<Box<str>>,
    line: Option<u32>,
    routine: Option<Box<str>>,
}

impl DbError {
    pub(crate) fn parse(fields: &mut ErrorFields<'_>) -> io::Result<DbError> {
        let mut severity = None;
        let mut parsed_severity = None;
        let mut code = None;
        let mut message = None;
        let mut detail = None;
        let mut hint = None;
        let mut normal_position = None;
        let mut internal_position = None;
        let mut internal_query = None;
        let mut where_ = None;
        let mut schema = None;
        let mut table = None;
        let mut column = None;
        let mut datatype = None;
        let mut constraint = None;
        let mut file = None;
        let mut line = None;
        let mut routine = None;

        while let Some(field) = fields.next()? {
            match field.type_() {
                b'S' => severity = Some(field.value().to_string().into_boxed_str()),
                b'C' => code = Some(SqlState::from_code(field.value())),
                b'M' => message = Some(field.value().to_string().into_boxed_str()),
                b'D' => detail = Some(field.value().to_string().into_boxed_str()),
                b'H' => hint = Some(field.value().to_string().into_boxed_str()),
                b'P' => {
                    normal_position = Some(field.value().parse::<u32>().map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "`P` field did not contain an integer",
                        )
                    })?);
                }
                b'p' => {
                    internal_position = Some(field.value().parse::<u32>().map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "`p` field did not contain an integer",
                        )
                    })?);
                }
                b'q' => internal_query = Some(field.value().to_owned()),
                b'W' => where_ = Some(field.value().to_string().into_boxed_str()),
                b's' => schema = Some(field.value().to_string().into_boxed_str()),
                b't' => table = Some(field.value().to_string().into_boxed_str()),
                b'c' => column = Some(field.value().to_string().into_boxed_str()),
                b'd' => datatype = Some(field.value().to_string().into_boxed_str()),
                b'n' => constraint = Some(field.value().to_string().into_boxed_str()),
                b'F' => file = Some(field.value().to_string().into_boxed_str()),
                b'L' => {
                    line = Some(field.value().parse::<u32>().map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "`L` field did not contain an integer",
                        )
                    })?);
                }
                b'R' => routine = Some(field.value().to_string().into_boxed_str()),
                b'V' => {
                    parsed_severity = Some(Severity::from_str(field.value()).ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "`V` field contained an invalid value",
                        )
                    })?);
                }
                _ => {}
            }
        }

        Ok(DbError {
            severity: severity
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "`S` field missing"))?,
            parsed_severity,
            code: code
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "`C` field missing"))?,
            message: message
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "`M` field missing"))?,
            detail,
            hint,
            position: match normal_position {
                Some(position) => Some(ErrorPosition::Original(position)),
                None => match internal_position {
                    Some(position) => Some(ErrorPosition::Internal {
                        position,
                        query: internal_query.ok_or_else(|| {
                            io::Error::new(
                                io::ErrorKind::InvalidInput,
                                "`q` field missing but `p` field present",
                            )
                        })?,
                    }),
                    None => None,
                },
            },
            where_,
            schema,
            table,
            column,
            datatype,
            constraint,
            file,
            line,
            routine,
        })
    }

    /// The field contents are ERROR, FATAL, or PANIC (in an error message),
    /// or WARNING, NOTICE, DEBUG, INFO, or LOG (in a notice message), or a
    /// localized translation of one of these.
    pub fn severity(&self) -> &str {
        &self.severity
    }

    /// A parsed, nonlocalized version of `severity`. (PostgreSQL 9.6+)
    pub fn parsed_severity(&self) -> Option<Severity> {
        self.parsed_severity
    }

    /// The SQLSTATE code for the error.
    pub fn code(&self) -> &SqlState {
        &self.code
    }

    /// The primary human-readable error message.
    ///
    /// This should be accurate but terse (typically one line).
    pub fn message(&self) -> &str {
        &self.message
    }

    /// An optional secondary error message carrying more detail about the
    /// problem.
    ///
    /// Might run to multiple lines.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    /// An optional suggestion what to do about the problem.
    ///
    /// This is intended to differ from `detail` in that it offers advice
    /// (potentially inappropriate) rather than hard facts. Might run to
    /// multiple lines.
    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }

    /// An optional error cursor position into either the original query string
    /// or an internally generated query.
    pub fn position(&self) -> Option<&ErrorPosition> {
        self.position.as_ref()
    }

    /// An indication of the context in which the error occurred.
    ///
    /// Presently this includes a call stack traceback of active procedural
    /// language functions and internally-generated queries. The trace is one
    /// entry per line, most recent first.
    pub fn where_(&self) -> Option<&str> {
        self.where_.as_deref()
    }

    /// If the error was associated with a specific database object, the name
    /// of the schema containing that object, if any. (PostgreSQL 9.3+)
    pub fn schema(&self) -> Option<&str> {
        self.schema.as_deref()
    }

    /// If the error was associated with a specific table, the name of the
    /// table. (Refer to the schema name field for the name of the table's
    /// schema.) (PostgreSQL 9.3+)
    pub fn table(&self) -> Option<&str> {
        self.table.as_deref()
    }

    /// If the error was associated with a specific table column, the name of
    /// the column.
    ///
    /// (Refer to the schema and table name fields to identify the table.)
    /// (PostgreSQL 9.3+)
    pub fn column(&self) -> Option<&str> {
        self.column.as_deref()
    }

    /// If the error was associated with a specific data type, the name of the
    /// data type. (Refer to the schema name field for the name of the data
    /// type's schema.) (PostgreSQL 9.3+)
    pub fn datatype(&self) -> Option<&str> {
        self.datatype.as_deref()
    }

    /// If the error was associated with a specific constraint, the name of the
    /// constraint.
    ///
    /// Refer to fields listed above for the associated table or domain.
    /// (For this purpose, indexes are treated as constraints, even if they
    /// weren't created with constraint syntax.) (PostgreSQL 9.3+)
    pub fn constraint(&self) -> Option<&str> {
        self.constraint.as_deref()
    }

    /// The file name of the source-code location where the error was reported.
    pub fn file(&self) -> Option<&str> {
        self.file.as_deref()
    }

    /// The line number of the source-code location where the error was
    /// reported.
    pub fn line(&self) -> Option<u32> {
        self.line
    }

    /// The name of the source-code routine reporting the error.
    pub fn routine(&self) -> Option<&str> {
        self.routine.as_deref()
    }
}

impl fmt::Display for DbError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}: {}", self.severity, self.message)?;
        if let Some(detail) = &self.detail {
            write!(fmt, "\nDETAIL: {}", detail)?;
        }
        if let Some(hint) = &self.hint {
            write!(fmt, "\nHINT: {}", hint)?;
        }
        Ok(())
    }
}

impl StdError for DbError {}

/// Represents the position of an error in a query.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ErrorPosition {
    /// A position in the original query.
    Original(u32),
    /// A position in an internally generated query.
    Internal {
        /// The byte position.
        position: u32,
        /// A query generated by the Postgres server.
        query: String,
    },
}

/// An error communicating with the Postgres server.
#[derive(Debug)]
pub enum Kind {
    /// An IO Error occurred.
    Io(io::Error),
    /// An unexpected message was received from postgres,
    UnexpectedMessage,
    /// An error occurred during the TLS handshake.
    Tls(Box<dyn StdError + Sync + Send>),
    /// An error occurred while converting Rust data to bytes to form a request.
    ToSql(usize, Box<dyn StdError + Sync + Send>),
    /// An error occurred while converting bytes received from postgres to Rust data.
    FromSql(usize, Box<dyn StdError + Sync + Send>),
    /// An error occurred with given column.
    Column(String),
    /// A upexpected number of parameters was given during the encoding of a prepared statement.
    Parameters(usize, usize),
    /// The connection is closed.
    Closed,
    /// An error was returned from postgres.
    Db(Box<DbError>),
    /// An error occurred during parsing a response.
    Parse(io::Error),
    /// An error occurred during encoding a request.
    Encode(io::Error),
    /// An error occurred during authentication.
    Authentication(Box<dyn StdError + Sync + Send>),
    /// An error occurred while parsing the config string.
    ConfigParse(Box<dyn StdError + Sync + Send>),
    /// A logical error occurred while trying to use a well formed config.
    Config(Box<dyn StdError + Sync + Send>),
    /// An error occurred while connecting to a server.
    #[cfg(feature = "runtime")]
    Connect(io::Error),
    /// A query returned an unexpected number of rows.
    RowCount {
        /// An indication for the expected number of rows.
        expected: RowCountCategory,
        /// An indication for the number of rows in the result set.
        got: RowCountCategory,
    },
    /// A timeout while waiting for the server.
    Timeout,
}

#[derive(Debug, Clone, Copy)]
/// A enum to be able to indicate the scenario for the row count error.
pub enum RowCountCategory {
    /// On
    One,
    /// An optional row.
    ZeroOrOne,
    /// More than one row.
    MoreThanOne,
    /// No result rows.
    Zero,
}

impl fmt::Display for RowCountCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RowCountCategory::One => f.write_str("one"),
            RowCountCategory::ZeroOrOne => f.write_str("zero or one"),
            RowCountCategory::MoreThanOne => f.write_str("more than one"),
            RowCountCategory::Zero => f.write_str("zero"),
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Io(err) => write!(f, "error communicating with the server: {err}"),
            Kind::UnexpectedMessage => f.write_str("unexpected message from server"),
            Kind::Tls(err) => write!(f, "error performing TLS handshake: {err}"),
            Kind::ToSql(idx, err) => write!(f, "error serializing parameter {idx}: {err}"),
            Kind::FromSql(idx, err) => write!(f, "error deserializing column {idx}: {err}"),
            Kind::Column(column) => write!(f, "invalid column `{column}`"),
            Kind::Parameters(real, expected) => {
                write!(f, "expected {expected} parameters but got {real}")
            }
            Kind::Closed => f.write_str("connection closed"),
            Kind::Db(err) => write!(f, "db error: {err}"),
            Kind::Parse(err) => write!(f, "error parsing response from server: {err}"),
            Kind::Encode(err) => write!(f, "error encoding message to server: {err}"),
            Kind::Authentication(err) => write!(f, "authentication error: {err}"),
            Kind::ConfigParse(err) => write!(f, "invalid connection string: {err}"),
            Kind::Config(err) => write!(f, "invalid configuration: {err}"),
            #[cfg(feature = "runtime")]
            Kind::Connect(err) => write!(f, "error connecting to server: {err}"),
            Kind::RowCount { expected, got } => write!(
                f,
                "query returned an unexpected number of rows, expected {expected}, got {got}",
            ),
            Kind::Timeout => f.write_str("timeout waiting for server"),
        }
    }
}

struct ErrorInner {
    kind: Kind,
    #[cfg(feature = "tracing-error")]
    span_trace: Option<tracing_error::SpanTrace>,
}

/// An error communicating with the Postgres server.
pub struct Error(Box<ErrorInner>);

impl fmt::Debug for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ds = fmt.debug_struct("Error");
        ds.field("kind", &self.0.kind);

        #[cfg(feature = "tracing-error")]
        ds.field("span_trace:", &self.0.span_trace);

        ds.finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0.kind)?;

        #[cfg(feature = "tracing-error")]
        {
            if f.alternate() {
                if let Some(span_trace) = self
                    .0
                    .span_trace
                    .as_ref()
                    .filter(|s| s.status() != tracing_error::SpanTraceStatus::EMPTY)
                {
                    write!(f, "\n\nSpanTrace:\n")?;
                    fmt::Display::fmt(&span_trace, f)?;
                }
            }
        }

        Ok(())
    }
}

impl StdError for Error {
    #[allow(trivial_casts)]
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.0.kind {
            Kind::Io(err) => Some(err as _),
            Kind::UnexpectedMessage => None,
            Kind::Tls(err) => Some(&**err as _),
            Kind::ToSql(_, err) => Some(&**err as _),
            Kind::FromSql(_, err) => Some(&**err as _),
            Kind::Column(_) => None,
            Kind::Parameters(..) => None,
            Kind::Closed => None,
            Kind::Db(err) => Some(&**err as _),
            Kind::Parse(err) => Some(err as _),
            Kind::Encode(err) => Some(err as _),
            Kind::Authentication(err) => Some(&**err as _),
            Kind::ConfigParse(err) => Some(&**err as _),
            Kind::Config(err) => Some(&**err as _),
            #[cfg(feature = "runtime")]
            Kind::Connect(err) => Some(err as _),
            Kind::RowCount { .. } => None,
            Kind::Timeout => None,
        }
    }
}

impl Error {
    /// Consumes the error, returning its cause.
    pub fn into_source(self) -> Option<Box<dyn StdError + Sync + Send>> {
        match self.0.kind {
            Kind::Io(err) => Some(Box::new(err)),
            Kind::UnexpectedMessage => None,
            Kind::Tls(err) => Some(err),
            Kind::ToSql(_, err) => Some(err),
            Kind::FromSql(_, err) => Some(err),
            Kind::Column(_) => None,
            Kind::Parameters(..) => None,
            Kind::Closed => None,
            Kind::Db(err) => Some(Box::new(err)),
            Kind::Parse(err) => Some(Box::new(err)),
            Kind::Encode(err) => Some(Box::new(err)),
            Kind::Authentication(err) => Some(err),
            Kind::ConfigParse(err) => Some(err),
            Kind::Config(err) => Some(err),
            #[cfg(feature = "runtime")]
            Kind::Connect(err) => Some(Box::new(err)),
            Kind::RowCount { .. } => None,
            Kind::Timeout => None,
        }
    }

    /// Returns the source of this error if it was a `DbError`.
    ///
    /// This is a simple convenience method.
    pub fn as_db_error(&self) -> Option<&DbError> {
        match &self.0.kind {
            Kind::Db(err) => Some(err),
            _ => None,
        }
    }

    /// Determines if the error was associated with closed connection.
    pub fn is_closed(&self) -> bool {
        matches!(self.0.kind, Kind::Closed)
    }

    /// Returns the SQLSTATE error code associated with the error.
    ///
    /// This is a convenience method that downcasts the cause to a `DbError` and returns its code.
    pub fn code(&self) -> Option<&SqlState> {
        self.as_db_error().map(DbError::code)
    }

    fn new(kind: Kind) -> Self {
        Self(Box::new(ErrorInner {
            kind,
            #[cfg(feature = "tracing-error")]
            span_trace: Some(tracing_error::SpanTrace::capture()),
        }))
    }

    /// Return the Kind
    pub fn kind(&self) -> &Kind {
        &self.0.kind
    }

    /// Return the Kind
    pub fn into_kind(self) -> Kind {
        self.0.kind
    }

    /// Return the captured SpanTrace. None is returned if the SpanTrace was already taken.
    #[cfg(feature = "tracing-error")]
    pub fn span_trace(&self) -> Option<&tracing_error::SpanTrace> {
        self.0.span_trace.as_ref()
    }

    /// Take ownership of the captured SpanTrace.
    ///
    /// None is returned if the SpanTrace was already taken.
    #[cfg(feature = "tracing-error")]
    pub fn take_span_trace(&mut self) -> Option<tracing_error::SpanTrace> {
        self.0.span_trace.take()
    }

    pub(crate) fn io(e: io::Error) -> Error {
        Error::new(Kind::Io(e))
    }

    pub(crate) fn unexpected_message() -> Error {
        Error::new(Kind::UnexpectedMessage)
    }

    pub(crate) fn tls(err: Box<dyn StdError + Sync + Send>) -> Error {
        Error::new(Kind::Tls(err))
    }

    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_sql(e: Box<dyn StdError + Sync + Send>, idx: usize) -> Error {
        Error::new(Kind::ToSql(idx, e))
    }

    pub(crate) fn from_sql(e: Box<dyn StdError + Sync + Send>, idx: usize) -> Error {
        Error::new(Kind::FromSql(idx, e))
    }
    pub(crate) fn column(column: String) -> Error {
        Error::new(Kind::Column(column))
    }

    pub(crate) fn parameters(real: usize, expected: usize) -> Error {
        Error::new(Kind::Parameters(real, expected))
    }

    pub(crate) fn closed() -> Error {
        Error::new(Kind::Closed)
    }

    pub(crate) fn db(error: ErrorResponseBody) -> Error {
        match DbError::parse(&mut error.fields()) {
            Ok(e) => Error::new(Kind::Db(Box::new(e))),
            Err(e) => Error::new(Kind::Parse(e)),
        }
    }

    pub(crate) fn parse(e: io::Error) -> Error {
        Error::new(Kind::Parse(e))
    }

    pub(crate) fn encode(e: io::Error) -> Error {
        Error::new(Kind::Encode(e))
    }

    pub(crate) fn authentication(e: Box<dyn StdError + Sync + Send>) -> Error {
        Error::new(Kind::Authentication(e))
    }

    pub(crate) fn config_parse(e: Box<dyn StdError + Sync + Send>) -> Error {
        Error::new(Kind::ConfigParse(e))
    }

    pub(crate) fn config(e: Box<dyn StdError + Sync + Send>) -> Error {
        Error::new(Kind::Config(e))
    }

    pub(crate) fn row_count(expected: RowCountCategory, got: RowCountCategory) -> Error {
        Error::new(Kind::RowCount { expected, got })
    }

    #[cfg(feature = "runtime")]
    pub(crate) fn connect(e: io::Error) -> Error {
        Error::new(Kind::Connect(e))
    }

    #[doc(hidden)]
    pub fn __private_api_timeout() -> Error {
        Error::new(Kind::Timeout)
    }
}
