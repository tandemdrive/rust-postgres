use crate::{FromSql, IsNull, ToSql, Type};
use bytes::{BufMut, BytesMut};
use serde_1::{Deserialize, Serialize};
use serde_json_1::{value::RawValue, Value};
use std::error::Error;
use std::fmt::Debug;
use std::io::Read;

/// A wrapper type to allow arbitrary `Serialize`/`Deserialize` types to convert to Postgres JSON values.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Json<T>(pub T);

impl<'a, T> FromSql<'a> for Json<T>
where
    T: Deserialize<'a>,
{
    fn from_sql(ty: &Type, mut raw: &'a [u8]) -> Result<Json<T>, Box<dyn Error + Sync + Send>> {
        if *ty == Type::JSONB {
            let mut b = [0; 1];
            raw.read_exact(&mut b)?;
            // We only support version 1 of the jsonb binary format
            if b[0] != 1 {
                return Err("unsupported JSONB encoding version".into());
            }
        }
        serde_json_1::de::from_slice(raw)
            .map(Json)
            .map_err(Into::into)
    }

    accepts!(JSON, JSONB);
}

impl<T> ToSql for Json<T>
where
    T: Serialize + Debug,
{
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        if *ty == Type::JSONB {
            out.put_u8(1);
        }
        serde_json_1::ser::to_writer(out.writer(), &self.0)?;
        Ok(IsNull::No)
    }

    accepts!(JSON, JSONB);
    to_sql_checked!();
}

impl<'a> FromSql<'a> for Value {
    fn from_sql(ty: &Type, raw: &[u8]) -> Result<Value, Box<dyn Error + Sync + Send>> {
        Json::<Value>::from_sql(ty, raw).map(|json| json.0)
    }

    accepts!(JSON, JSONB);
}

impl ToSql for Value {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        Json(self).to_sql(ty, out)
    }

    accepts!(JSON, JSONB);
    to_sql_checked!();
}

impl<'a> FromSql<'a> for &'a RawValue {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<&'a RawValue, Box<dyn Error + Sync + Send>> {
        Json::<&'a RawValue>::from_sql(ty, raw).map(|json| json.0)
    }

    accepts!(JSON, JSONB);
}

impl<'a> ToSql for &'a RawValue {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        Json(self).to_sql(ty, out)
    }

    accepts!(JSON, JSONB);
    to_sql_checked!();
}

impl<'a> FromSql<'a> for Box<RawValue> {
    fn from_sql(ty: &Type, raw: &[u8]) -> Result<Box<RawValue>, Box<dyn Error + Sync + Send>> {
        Json::<Box<RawValue>>::from_sql(ty, raw).map(|json| json.0)
    }

    accepts!(JSON, JSONB);
}

impl ToSql for Box<RawValue> {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        Json(self).to_sql(ty, out)
    }

    accepts!(JSON, JSONB);
    to_sql_checked!();
}
