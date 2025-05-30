use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use super::{ConnectionBuilder, ConnectionPool, Protocol, Connection, Result}; 
use super::super::http::response::HttpResponse;  
use crate::pool::Pool;

const TEST_HTTP_SERVER: &str = "fds.rs";
const TEST_HTTPS_SERVER: &str = "fds.rs";

async fn send_http_request(conn: &mut Connection) -> Result<String> {
    let request = "GET / HTTP/1.1\r\nHost: fds.rs\r\nConnection: close\r\n\r\n";
    conn.write_all(request.as_bytes()).await?;
    
    let mut response = Vec::new();
    conn.read_to_end(&mut response).await?;
    Ok(String::from_utf8_lossy(&response).into())
} 

#[tokio::test]
async fn test_https_connection() {
    let builder = ConnectionBuilder::new(TEST_HTTPS_SERVER, 443)
        .protocol(Protocol::HTTP)
        .tls(true);

    let mut conn = builder.connect().await.unwrap();
    let response = send_http_request(&mut conn).await.unwrap();
    
    // assert!(response.contains("HTTP/1.1 200 OK"));
    // assert!(response.contains("\"url\": \"https://httpbin.org/get\"")); 

    println!("Response: {}", response); // Debugging output 
} 

#[tokio::test]
async fn test_connection_pool() {
    let builder = ConnectionBuilder::new(TEST_HTTP_SERVER, 80)
        .protocol(Protocol::HTTP);
    
    let pool = ConnectionPool::new(builder, 5)
        .min_size(2)
        .max_idle_time(Duration::from_secs(30)); 

    pool.initialize()
        .await
        .unwrap(); 

    // // Test pool capacity
    // let mut connections = Vec::new();
    // for _ in 0..5 {
    //     connections.push(pool.get().await.unwrap());
    // }
    
    // Should fail on 6th attempt
   //  assert!(pool.get().await.is_err());
    
    // Release connections
    // for conn in connections {
    //     pool.release(conn).await;
    // }

    // Verify reuse
    // let start_count = pool.connections.lock().await.len();
    // let mut conn = pool.get().await.unwrap();
    // let _ = send_http_request(&mut *conn.connection.lock().await).await;
    
    // assert_eq!(pool.connections.lock().await.len(), start_count - 1);
} 

#[tokio::test]
async fn test_connection_pool_trait() {
    let builder = ConnectionBuilder::new(TEST_HTTP_SERVER, 80)
        .protocol(Protocol::HTTP);
    let pool = ConnectionPool::new(builder, 2)
        .min_size(1)
        .max_idle_time(Duration::from_secs(30));

    pool.initialize().await.unwrap();

    // Test get and release via Pool trait
    let item = <ConnectionPool as Pool>::get(&pool).await.expect("Pool::get failed");
    <ConnectionPool as Pool>::release(&pool, item).await;
} 
