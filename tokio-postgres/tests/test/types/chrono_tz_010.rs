use std::str::FromStr;

use crate::types::test_type;

#[tokio::test]
async fn test_tz() {
    fn make_check(time: &str) -> (Option<chrono_tz_010::Tz>, &str) {
        (
            Some(chrono_tz_010::Tz::from_str(&time[1..time.len() - 1]).unwrap()),
            time,
        )
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
