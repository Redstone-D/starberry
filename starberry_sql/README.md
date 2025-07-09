# starberry_sql

`starberry_sql` is an asynchronous SQL client and query builder designed for the Starberry ecosystem. It provides a simple, type-safe API to execute queries, map rows to structs, manage transactions, and handle connection pooling.

## Features

- Asynchronous connections via **tokio**
- Secure TLS support using **rustls**
- Safe parameter binding with the `Encode` trait
- Flexible query builder via `SqlQuery`
- Automatic row-to-struct mapping with the `FromRow` trait
- Convenient result handling through `QueryResult` methods
- Connection pooling with `SqlPool`
- Transaction support: begin, commit, rollback
- Batch execution and prepared statements
- Customizable SQL context via `SqlContext`

## Installation

Add `starberry_sql` to your `Cargo.toml`:

```toml
[dependencies]
starberry_sql = "0.6.4"
```

Then import in your code:

```rust
use starberry_sql::*;
```

## Quick Start

### Establish a Connection

```rust
use starberry_sql::{DbConnectionBuilder, DbError, SslMode};

#[tokio::main]
async fn main() -> Result<(), DbError> {
    let mut conn = DbConnectionBuilder::new("127.0.0.1", 5432)
        .ssl_mode(SslMode::Disable)
        .database("postgres")
        .username("postgres")
        .password("secret")
        .connect()
        .await?;
    Ok(())
}
```

### Execute a Simple Query

```rust
let rows = SqlQuery::new("SELECT 1 AS a, 'foo' AS b")
    .fetch_all(&mut conn)
    .await?;
for row in rows {
    println!("a = {}", row.get("a").unwrap());
}
```

### Insert Data with Parameter Binding

```rust
let count = SqlQuery::new("INSERT INTO users (name, age) VALUES ($1, $2)")
    .bind("Alice")
    .bind(30)
    .execute(&mut conn)
    .await?;
println!("Inserted {} rows", count);
```

### Map Rows to Your Structs

```rust
use std::collections::HashMap;
use starberry_sql::{FromRow, DbError};

#[derive(Debug)]
struct User { id: i32, name: String }

impl FromRow for User {
    fn from_row(row: &HashMap<String, String>) -> Result<Self, DbError> {
        Ok(User {
            id: row.get("id").unwrap().parse().unwrap(),
            name: row.get("name").unwrap().to_string(),
        })
    }
}

let users: Vec<User> = SqlQuery::new("SELECT id, name FROM users")
    .fetch_all_as(&mut conn)
    .await?;
```

### Connection Pooling

```rust
let builder = DbConnectionBuilder::new("127.0.0.1", 5432)
    .ssl_mode(SslMode::Disable)
    .database("postgres")
    .username("postgres")
    .password("secret");
let pool = SqlPool::new(builder, 5);

let row = SqlQuery::new("SELECT COUNT(*) AS count")
    .fetch_one_pool(&pool)
    .await?;
```

### Transactions

```rust
conn.begin_transaction().await?;
// perform multiple statements
conn.commit_transaction().await?;
```

### Batch Execution and Prepared Statements

```rust
let total = conn.batch_execute(vec![
    ("INSERT INTO tx_test (id, name) VALUES ($1, $2)", vec!["1".to_string(), "One".to_string()]),
    ("INSERT INTO tx_test (id, name) VALUES ($1, $2)", vec!["2".to_string(), "Two".to_string()]),
])
.await?
.row_count();

let stmt = conn.prepare_statement("INSERT INTO tx_test (id, name) VALUES ($1, $2)")
    .await?;
let result = conn.execute_prepared(&stmt, vec!["5".to_string(), "Five".to_string()])
    .await?;
```

## Modules Overview

- **`DbConnectionBuilder`**: Configure and establish database connections.
- **`SqlQuery`**: Build and execute queries with parameter binding.
- **`SqlPool`**: Manage a pool of connections for concurrent usage.
- **`SqlContext`**: Customize SQL execution context and settings.
- **`Encode`** trait: Convert Rust types into SQL-escaped literals.
- **`FromRow`** trait: Map a row (as `HashMap<String, String>`) into a struct.
- **`QueryResult`**: Handle query outcomes (`Rows`, `Count`, `Empty`, `Error`).
- **`DbError`**: Unified error type for connection and query operations.

## License

`starberry_sql` is released under the MIT License. See [LICENSE](../LICENSE.txt) for details. 