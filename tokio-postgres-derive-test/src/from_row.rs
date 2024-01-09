use futures_util::FutureExt;

use tokio_postgres::FromRow;
use tokio_postgres::{Client, NoTls};

#[tokio::test]
async fn query_all_as() {
    #[derive(Debug, PartialEq)]
    struct Age(i32);

    #[derive(Debug, PartialEq, Default)]
    struct NonSqlType;

    impl From<i32> for Age {
        fn from(value: i32) -> Self {
            Self(value)
        }
    }

    #[derive(FromRow)]
    struct Person<A, S> {
        name: String,
        #[from_row(from = "i32")]
        age: A,
        #[from_row(skip)]
        skip_this_column: S,
    }

    let client = connect("user=postgres host=localhost port=5433").await;
    client
        .batch_execute(
            "CREATE TEMPORARY TABLE person (
                id serial,
                name text,
                age integer
            );
            INSERT INTO person (name, age) VALUES ('steven', 18);
            ",
        )
        .await
        .unwrap();

    let users: Vec<Person<Age, NonSqlType>> = client
        .query_as("SELECT name, age FROM person", &[])
        .await
        .unwrap();

    assert_eq!(users.len(), 1);
    let user = users.get(0).unwrap();
    assert_eq!(user.name, "steven");
    assert_eq!(user.age, Age(18));
    assert_eq!(user.skip_this_column, NonSqlType);
}

async fn connect(s: &str) -> Client {
    let (client, connection) = tokio_postgres::connect(s, NoTls).await.unwrap();
    let connection = connection.map(|e| e.unwrap());
    tokio::spawn(connection);

    client
}
