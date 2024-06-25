use crate::{FromSql, ToSql};

impl<'a> FromSql<'a> for chrono_tz_09::Tz {
    fn from_sql(
        ty: &crate::Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let string = <&str>::from_sql(ty, raw)?;
        Ok(string.parse()?)
    }

    fn accepts(ty: &crate::Type) -> bool {
        <&str as FromSql>::accepts(ty)
    }
}

impl ToSql for chrono_tz_09::Tz {
    fn to_sql(
        &self,
        ty: &crate::Type,
        out: &mut bytes::BytesMut,
    ) -> Result<crate::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        let string = self.to_string();
        string.to_sql(ty, out)
    }

    fn accepts(ty: &crate::Type) -> bool {
        <&str as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}
