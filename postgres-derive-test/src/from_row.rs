use postgres;
use postgres::{Client, NoTls};
use postgres_types::FromRow;
// Because of the `FromRow` trait, `tokio-postgres` is needed as dependency.
// I don't see a different solution without restructuring `rust-postgres`.
// And `postgres` is already using `tokio-postgres` underneath, so what's the issue :-)
use tokio_postgres;

#[test]
fn query_all_as() {
    #[derive(FromRow)]
    struct User {
        name: String,
        age: i32,
    }

    let mut client = Client::connect("user=postgres host=localhost port=5433", NoTls).unwrap();

    client
        .batch_execute(
            "CREATE TEMPORARY TABLE user2 (
                id serial,
                name text,
                age integer
            );
            INSERT INTO user2 (name, age) VALUES ('steven', 18);
            ",
        )
        .unwrap();

    let users: Vec<User> = client
        .query_all_as("SELECT name, age FROM user2", &[])
        .unwrap();

    assert_eq!(users.len(), 1);
    let user = users.get(0).unwrap();
    assert_eq!(user.name, "steven");
    assert_eq!(user.age, 18)
}
