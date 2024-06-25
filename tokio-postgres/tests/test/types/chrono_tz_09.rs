use std::str::FromStr;

use crate::types::test_type;

#[tokio::test]
async fn test_tz() {
    fn make_check(time: &str) -> (Option<chrono_tz_09::Tz>, &str) {
        (Some(chrono_tz_09::Tz::from_str(time).unwrap()), time)
    }
    test_type(
        "VARCHAR",
        &[
            make_check("'Antarctica/South_Pole'"),
            make_check("'Europe/Amsterdam'"),
            (None, "NULL"),
        ],
    )
    .await;
}
