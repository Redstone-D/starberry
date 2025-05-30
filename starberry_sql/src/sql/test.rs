use super::*;
use std::collections::HashMap;
use starberry_core::pool::Pool;

#[test]
fn test_encode_primitives() {
    // i32
    assert_eq!(1.encode().unwrap(), "1".to_string());
    // &str
    assert_eq!("foo".encode().unwrap(), "'foo'".to_string());
    assert_eq!("a'b".encode().unwrap(), "'a''b'".to_string());
    // String
    assert_eq!("bar".to_string().encode().unwrap(), "'bar'".to_string());
    // bool
    assert_eq!(true.encode().unwrap(), "TRUE".to_string());
    assert_eq!(false.encode().unwrap(), "FALSE".to_string());
    // Option<T>
    let some_str: Option<&str> = Some("baz");
    assert_eq!(some_str.encode().unwrap(), "'baz'".to_string());
    let none_int: Option<i32> = None;
    assert_eq!(none_int.encode().unwrap(), "NULL".to_string());
}

#[derive(Debug, PartialEq)]
struct TestRow {
    id: i32,
    name: String,
}

impl FromRow for TestRow {
    fn from_row(row: &HashMap<String, String>) -> Result<Self, DbError> {
        let id = row.get("id")
            .ok_or_else(|| DbError::QueryError("Missing id".into()))?
            .parse::<i32>().map_err(|e| DbError::QueryError(e.to_string()))?;
        let name = row.get("name")
            .ok_or_else(|| DbError::QueryError("Missing name".into()))?
            .clone();
        Ok(TestRow { id, name })
    }
}

#[test]
fn test_from_row_success_and_error() {
    let mut row_map = HashMap::new();
    row_map.insert("id".to_string(), "10".to_string());
    row_map.insert("name".to_string(), "alice".to_string());
    let row = TestRow::from_row(&row_map).unwrap();
    assert_eq!(row, TestRow { id: 10, name: "alice".to_string() });
    row_map.remove("id");
    assert!(TestRow::from_row(&row_map).is_err());
}

#[test]
fn test_query_result_methods() {
    let mut row_map = HashMap::new();
    row_map.insert("k".to_string(), "v".to_string());
    let qr_rows = QueryResult::Rows(vec![row_map.clone()]);
    assert_eq!(qr_rows.row_count(), 1);
    assert_eq!(qr_rows.first_row(), Some(&row_map));
    let qr_count = QueryResult::Count(5);
    assert_eq!(qr_count.row_count(), 5);
    let qr_empty = QueryResult::Empty;
    assert_eq!(qr_empty.row_count(), 0);
    let qr_error = QueryResult::Error(DbError::OtherError("e".to_string()));
    assert_eq!(qr_error.row_count(), 0);
}

#[tokio::test]
async fn test_sql_query_fetch_methods() {
    // Setup connection
    let mut conn = DbConnectionBuilder::new("127.0.0.1", 5432)
        .ssl_mode(SslMode::Disable)
        .database("postgres")
        .username("postgres")
        .password("JerrySu5379")
        .connect().await.expect("Failed to connect to Postgres");
    // fetch_all and fetch_one
    let rows = SqlQuery::new("SELECT 1 AS a, 'foo' AS b")
        .fetch_all(&mut conn).await.expect("fetch_all failed");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get("a"), Some(&"1".to_string()));
    assert_eq!(rows[0].get("b"), Some(&"foo".to_string()));
    let row = SqlQuery::new("SELECT 2 AS a, 'bar' AS b")
        .fetch_one(&mut conn).await.expect("fetch_one failed");
    assert_eq!(row.get("a"), Some(&"2".to_string()));
    assert_eq!(row.get("b"), Some(&"bar".to_string()));
    // fetch_all_as and fetch_one_as
    let items: Vec<TestRow> = SqlQuery::new("SELECT 10 AS id, 'alice' AS name")
        .fetch_all_as(&mut conn).await.expect("fetch_all_as failed");
    assert_eq!(items, vec![TestRow { id: 10, name: "alice".to_string() }]);
    let item: TestRow = SqlQuery::new("SELECT 20 AS id, 'bob' AS name")
        .fetch_one_as(&mut conn).await.expect("fetch_one_as failed");
    assert_eq!(item, TestRow { id: 20, name: "bob".to_string() });
    // execute non-select via execute (create and drop temp table)
    let _ = SqlQuery::new("CREATE TEMP TABLE temp_test (id INT)")
        .execute(&mut conn).await.expect("create temp table failed");
    let cnt = SqlQuery::new("INSERT INTO temp_test (id) VALUES ($1), ($2)")
        .bind(30).bind(40)
        .execute(&mut conn).await.expect("insert failed");
    assert_eq!(cnt, 2);
    let select_rows = SqlQuery::new("SELECT id FROM temp_test ORDER BY id")
        .fetch_all(&mut conn).await.expect("select failed");
    assert_eq!(select_rows.len(), 2);
    let _ = SqlQuery::new("DROP TABLE temp_test")
        .execute(&mut conn).await.expect("drop table failed");
}

#[tokio::test]
async fn test_sql_pool_methods() {
    let builder = DbConnectionBuilder::new("127.0.0.1", 5432)
        .ssl_mode(SslMode::Disable)
        .database("postgres")
        .username("postgres")
        .password("JerrySu5379");
    let pool = SqlPool::new(builder, 5);
    // fetch_all_pool and fetch_one_pool
    let rows = SqlQuery::new("SELECT 3 AS a, 'z' AS b")
        .fetch_all_pool(&pool).await.expect("fetch_all_pool failed");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get("a"), Some(&"3".to_string()));
    assert_eq!(rows[0].get("b"), Some(&"z".to_string()));
    let row = SqlQuery::new("SELECT 4 AS a, 'y' AS b")
        .fetch_one_pool(&pool).await.expect("fetch_one_pool failed");
    assert_eq!(row.get("a"), Some(&"4".to_string()));
    assert_eq!(row.get("b"), Some(&"y".to_string()));
    // execute_pool
    let cnt = SqlQuery::new("CREATE TABLE IF NOT EXISTS temp_pool (id INT)")
        .execute_pool(&pool).await.expect("create temp table failed");
    assert_eq!(cnt, 0);
    let cnt2 = SqlQuery::new("INSERT INTO temp_pool (id) VALUES ($1), ($2)")
        .bind(1).bind(2)
        .execute_pool(&pool).await.expect("insert failed");
    assert_eq!(cnt2, 2);
    let fetched = SqlQuery::new("SELECT id FROM temp_pool ORDER BY id")
        .fetch_all_pool(&pool).await.expect("select failed");
    assert_eq!(fetched.len(), 2);
    let _ = SqlQuery::new("DROP TABLE IF EXISTS temp_pool")
        .execute_pool(&pool).await.expect("drop table failed");
}

#[tokio::test]
async fn test_batch_execute_and_transactions_and_prepare() {
    // Setup connection
    let mut conn = DbConnectionBuilder::new("127.0.0.1", 5432)
        .ssl_mode(SslMode::Disable)
        .database("postgres")
        .username("postgres")
        .password("JerrySu5379")
        .connect().await.expect("Failed to connect to Postgres");

    // Clean up any existing test table
    let _ = SqlQuery::new("DROP TABLE IF EXISTS tx_test").execute(&mut conn).await;

    // Create table
    SqlQuery::new("CREATE TABLE tx_test (id INT, name TEXT)")
        .execute(&mut conn).await.expect("Create table failed");

    // Batch execute
    let total = conn.batch_execute(vec![
        ("INSERT INTO tx_test (id, name) VALUES ($1, $2)", vec!["1".to_string(), "One".to_string()]),
        ("INSERT INTO tx_test (id, name) VALUES ($1, $2)", vec!["2".to_string(), "Two".to_string()]),
    ]).await.expect("batch_execute failed").row_count();
    assert_eq!(total, 2);

    // Transaction rollback
    conn.begin_transaction().await.expect("begin failed");
    let _ = conn.batch_execute(vec![
        ("INSERT INTO tx_test (id, name) VALUES ($1, $2)", vec!["3".to_string(), "Three".to_string()]),
    ]).await.expect("batch_execute inside tx failed");
    conn.rollback_transaction().await.expect("rollback failed");
    let rows_r = SqlQuery::new("SELECT id FROM tx_test WHERE id=3")
        .fetch_all(&mut conn).await.expect("select after rollback failed");
    assert_eq!(rows_r.len(), 0);

    // Transaction commit
    conn.begin_transaction().await.expect("begin failed");
    let _ = conn.batch_execute(vec![
        ("INSERT INTO tx_test (id, name) VALUES ($1, $2)", vec!["4".to_string(), "Four".to_string()]),
    ]).await.expect("batch_execute inside tx failed");
    conn.commit_transaction().await.expect("commit failed");
    let rows_c = SqlQuery::new("SELECT id FROM tx_test WHERE id=4")
        .fetch_all(&mut conn).await.expect("select after commit failed");
    assert_eq!(rows_c.len(), 1);

    // Prepare and execute_prepared
    let stmt = conn.prepare_statement("INSERT INTO tx_test (id, name) VALUES ($1, $2)")
        .await.expect("prepare failed");
    let r = conn.execute_prepared(&stmt, vec!["5".to_string(), "Five".to_string()])
        .await.expect("execute_prepared failed");
    assert_eq!(r.row_count(), 1);
    let rows_p = SqlQuery::new("SELECT id, name FROM tx_test WHERE id>=4 ORDER BY id")
        .fetch_all(&mut conn).await.expect("select after prepare failed");
    let ids: Vec<String> = rows_p.iter().map(|r| r.get("id").cloned().unwrap()).collect();
    assert_eq!(ids, vec!["4".to_string(), "5".to_string()]);

    // Clean up
    SqlQuery::new("DROP TABLE tx_test").execute(&mut conn).await.expect("drop table failed");
}

#[tokio::test]
async fn test_sqlpool_trait() {
    // Create a small pool
    let builder = DbConnectionBuilder::new("127.0.0.1", 5432)
        .ssl_mode(SslMode::Disable)
        .database("postgres")
        .username("postgres")
        .password("JerrySu5379");
    let pool = SqlPool::new(builder, 2);

    // Acquire and release via Pool trait
    let mut item = <SqlPool as Pool>::get(&pool).await.expect("Pool::get failed");
    // Ensure we can access the inner connection
    let _conn_ref = item.connection();
    <SqlPool as Pool>::release(&pool, item).await;
} 