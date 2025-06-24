#[tokio::test]
async fn test_https_connection() { 
    use super::{Connection, ConnectionBuilder, Protocol, Result};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[allow(dead_code)] 
    const TEST_HTTP_SERVER: &str = "fds.rs";
    const TEST_HTTPS_SERVER: &str = "fds.rs";

    async fn send_http_request(conn: &mut Connection) -> Result<String> {
        let request = "GET / HTTP/1.1\r\nHost: fds.rs\r\nConnection: close\r\n\r\n";
        conn.write_all(request.as_bytes()).await?;

        let mut response = Vec::new();
        conn.read_to_end(&mut response).await?;
        Ok(String::from_utf8_lossy(&response).into())
    }

    let builder = ConnectionBuilder::new(TEST_HTTPS_SERVER, 443)
        .protocol(Protocol::HTTP)
        .tls(true);

    let mut conn = builder.connect().await.unwrap();
    let response = send_http_request(&mut conn).await.unwrap();

    // assert!(response.contains("HTTP/1.1 200 OK"));
    // assert!(response.contains("\"url\": \"https://httpbin.org/get\""));

    println!("Response: {}", response); // Debugging output 
}
