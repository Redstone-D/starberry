use std::time::Duration;
use starberry_core::connection::{Protocol, ConnectionBuilder, Connection as GenericConnection};
use super::error::DbError;
use md5;
use starberry_lib::random_alphanumeric_string;
use base64::{engine::general_purpose, Engine as _};
use ring::{digest, hmac, pbkdf2};
use std::num::NonZeroU32;
use async_trait::async_trait;
use starberry_core::connection::Tx;

/// Represents PostgreSQL SSL mode options for connection.
#[derive(Debug, Clone, PartialEq)]
pub enum SslMode {
    Disable,  // No SSL
    Prefer,   // Try SSL, fall back to non-SSL
    Require,  // Require SSL
    VerifyCa, // Require SSL and verify server certificate
    VerifyFull, // Require SSL, verify server certificate and hostname
}

/// Represents a database connection configuration with PostgreSQL specifics.
#[derive(Debug, Clone)]
pub struct DbConnectionBuilder {
    host: String,
    port: u16,
    protocol: Protocol,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
    max_connection_time: Option<Duration>,
    query_timeout: Option<Duration>,
    ssl_mode: Option<SslMode>,
    ssl_cert: Option<String>,  // Path to client certificate
    ssl_key: Option<String>,   // Path to client private key
    ssl_root_cert: Option<String>,  // Path to server CA certificate
}

impl DbConnectionBuilder {
    /// Creates a new database connection builder with the specified host and port.
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            protocol: Protocol::Postgres, // Default to Postgres
            database: Some("postgres".to_string()), // Default database
            username: None,
            password: None,
            max_connection_time: None,
            query_timeout: None,
            ssl_mode: None,
            ssl_cert: None,
            ssl_key: None,
            ssl_root_cert: None,
        }
    }

    /// Sets the protocol for the database connection.
    pub fn protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = protocol;
        self
    }

    /// Sets the database name for the connection.
    pub fn database(mut self, database: &str) -> Self {
        self.database = Some(database.to_string());
        self
    }

    /// Sets the username for the database connection.
    pub fn username(mut self, username: &str) -> Self {
        self.username = Some(username.to_string());
        self
    }

    /// Sets the password for the database connection.
    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_string());
        self
    }

    /// Sets the maximum connection time for the database connection.
    pub fn max_connection_time(mut self, duration: Duration) -> Self {
        self.max_connection_time = Some(duration);
        self
    }

    /// Sets the query timeout for the database connection.
    pub fn query_timeout(mut self, duration: Duration) -> Self {
        self.query_timeout = Some(duration);
        self
    }

    /// Sets the SSL mode for PostgreSQL connections.
    pub fn ssl_mode(mut self, mode: SslMode) -> Self {
        self.ssl_mode = Some(mode);
        self
    }

    /// Sets the path to the client SSL certificate for PostgreSQL connections.
    pub fn ssl_cert(mut self, cert_path: &str) -> Self {
        self.ssl_cert = Some(cert_path.to_string());
        self
    }

    /// Sets the path to the client SSL private key for PostgreSQL connections.
    pub fn ssl_key(mut self, key_path: &str) -> Self {
        self.ssl_key = Some(key_path.to_string());
        self
    }

    /// Sets the path to the server CA certificate for PostgreSQL connections.
    pub fn ssl_root_cert(mut self, root_cert_path: &str) -> Self {
        self.ssl_root_cert = Some(root_cert_path.to_string());
        self
    }

    /// Attempts to establish a connection to the database with PostgreSQL specifics.
    pub async fn connect(&self) -> Result<DbConnection, DbError> {
        // Use the generic ConnectionBuilder for TCP/TLS and handshake
        let mut builder = ConnectionBuilder::new(&self.host, self.port)
            .protocol(Protocol::Postgres)
            .tls(!matches!(self.ssl_mode, Some(SslMode::Disable)));
        if let Some(db) = &self.database {
            builder = builder.database(db);
        }
        if let Some(user) = &self.username {
            if let Some(pw) = &self.password {
                builder = builder.credentials(user, pw);
            }
        }
        // Establish connection and map errors
        // Raw connection
        let mut conn = builder.connect().await.map_err(|e| DbError::ConnectionError(e.to_string()))?;
        // Perform PostgreSQL startup handshake
        {
            use tokio::io::{AsyncWriteExt, AsyncReadExt};
            // Build StartupMessage (protocol version 3.0, parameters)
            let mut body = Vec::new();
            // Protocol version number
            body.extend_from_slice(&196608u32.to_be_bytes());
            // Include application_name for better logging/auditing
            body.extend_from_slice(b"application_name"); body.push(0);
            body.extend_from_slice(b"starberry_core"); body.push(0);
            if let Some(user) = &self.username {
                body.extend_from_slice(b"user"); body.push(0);
                body.extend_from_slice(user.as_bytes()); body.push(0);
            }
            if let Some(db) = &self.database {
                body.extend_from_slice(b"database"); body.push(0);
                body.extend_from_slice(db.as_bytes()); body.push(0);
            }
            // Terminate parameters list
            body.push(0);
            // Prepend length
            let msg_len = (body.len() + 4) as u32;
            let mut startup_msg = Vec::new();
            startup_msg.extend_from_slice(&msg_len.to_be_bytes());
            startup_msg.extend_from_slice(&body);
            // Send StartupMessage
            conn.write_all(&startup_msg).await.map_err(|e| DbError::ConnectionError(e.to_string()))?;
            conn.flush().await.map_err(|e| DbError::ConnectionError(e.to_string()))?;
            // Initialize SCRAM state
            let mut scram_client_first_bare: Option<String> = None;
            let mut scram_client_nonce: Option<String> = None;
            let mut scram_server_key: Option<Vec<u8>> = None;
            let mut scram_auth_msg: Option<String> = None;
            // Read and process server responses until ReadyForQuery
            loop {
                let mut tag = [0u8];
                conn.read_exact(&mut tag).await.map_err(|e| DbError::ProtocolError(e.to_string()))?;
                let mut len_buf = [0u8; 4];
                conn.read_exact(&mut len_buf).await.map_err(|e| DbError::ProtocolError(e.to_string()))?;
                let payload_len = u32::from_be_bytes(len_buf);
                let mut payload = vec![0u8; (payload_len - 4) as usize];
                conn.read_exact(&mut payload).await.map_err(|e| DbError::ProtocolError(e.to_string()))?;
                match tag[0] {
                    b'R' => {
                        // Authentication request
                        let code = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
                        if code == 0 {
                            // AuthenticationOk, nothing to do
                        } else if code == 3 {
                            // CleartextPassword
                            if let Some(pw) = &self.password {
                                // Send PasswordMessage ('p')
                                let mut pwd_body = pw.as_bytes().to_vec(); pwd_body.push(0);
                                let pwd_len = (pwd_body.len() + 4) as u32;
                                let mut pwd_msg = Vec::new();
                                pwd_msg.push(b'p');
                                pwd_msg.extend_from_slice(&pwd_len.to_be_bytes());
                                pwd_msg.extend_from_slice(&pwd_body);
                                conn.write_all(&pwd_msg).await.map_err(|e| DbError::ProtocolError(e.to_string()))?;
                                conn.flush().await.map_err(|e| DbError::ProtocolError(e.to_string()))?;
                                continue;
                            } else {
                                return Err(DbError::ConnectionError("Password required".to_string()));
                            }
                        } else if code == 5 {
                            // MD5Password
                            if let (Some(pw), Some(user)) = (&self.password, &self.username) {
                                let salt = &payload[4..8];
                                // First MD5: MD5(password + username)
                                let k = format!("{}{}", pw, user);
                                let digest1 = md5::compute(k);
                                let hex1 = format!("{:x}", digest1);
                                // Second MD5: MD5(hex1 + salt)
                                let mut data = Vec::with_capacity(hex1.len() + salt.len());
                                data.extend_from_slice(hex1.as_bytes());
                                data.extend_from_slice(salt);
                                let digest2 = md5::compute(&data);
                                let md5pwd = format!("md5{:x}", digest2);
                                // Send PasswordMessage ('p')
                                let mut pwd_body = md5pwd.as_bytes().to_vec();
                                pwd_body.push(0);
                                let pwd_len = (pwd_body.len() + 4) as u32;
                                let mut pwd_msg = Vec::new();
                                pwd_msg.push(b'p');
                                pwd_msg.extend_from_slice(&pwd_len.to_be_bytes());
                                pwd_msg.extend_from_slice(&pwd_body);
                                conn.write_all(&pwd_msg).await.map_err(|e| DbError::ProtocolError(e.to_string()))?;
                                conn.flush().await.map_err(|e| DbError::ProtocolError(e.to_string()))?;
                                continue;
                            } else {
                                return Err(DbError::ConnectionError("Username and password required for MD5 authentication".to_string()));
                            }
                        } else if code == 10 {
                            // AuthenticationSASL (SCRAM-SHA-256)
                            if let (Some(user), Some(pw)) = (&self.username, &self.password) {
                                // Parse supported mechanisms
                                let mut mechs = Vec::new();
                                let mut pos = 4;
                                while pos < payload.len() {
                                    if payload[pos] == 0 { break; }
                                    let end = payload[pos..].iter().position(|&b| b == 0).unwrap();
                                    let mech = String::from_utf8_lossy(&payload[pos..pos+end]).to_string();
                                    mechs.push(mech);
                                    pos += end + 1;
                                }
                                if !mechs.iter().any(|m| m == "SCRAM-SHA-256") {
                                    return Err(DbError::ProtocolError("SCRAM-SHA-256 not supported by server".to_string()));
                                }
                                // Client-first-message
                                let client_nonce = random_alphanumeric_string(24);
                                scram_client_nonce = Some(client_nonce.clone());
                                let client_first_bare = format!("n={},r={}", user, client_nonce);
                                scram_client_first_bare = Some(client_first_bare.clone());
                                let gs2 = "n,,";
                                let client_first_msg = format!("{}{}", gs2, client_first_bare);
                                // Build SASLInitialResponse
                                let mut body2 = Vec::new();
                                body2.extend_from_slice(b"SCRAM-SHA-256"); body2.push(0);
                                body2.extend_from_slice(&(client_first_msg.len() as u32).to_be_bytes());
                                body2.extend_from_slice(client_first_msg.as_bytes());
                                let len2 = (body2.len() + 4) as u32;
                                let mut msg2 = Vec::new();
                                msg2.push(b'p');
                                msg2.extend_from_slice(&len2.to_be_bytes());
                                msg2.extend_from_slice(&body2);
                                conn.write_all(&msg2).await.map_err(|e| DbError::ProtocolError(e.to_string()))?;
                                conn.flush().await.map_err(|e| DbError::ProtocolError(e.to_string()))?;
                                continue;
                            } else {
                                return Err(DbError::ConnectionError("Username and password required for SCRAM authentication".to_string()));
                            }
                        } else if code == 11 {
                            // AuthenticationSASLContinue (SCRAM server-first)
                            if let (Some(client_first_bare), Some(_client_nonce)) = (scram_client_first_bare.clone(), scram_client_nonce.clone()) {
                                let server_first = String::from_utf8(payload[4..].to_vec()).map_err(|e| DbError::ProtocolError(e.to_string()))?;
                                // Parse server-first-message
                                let mut server_nonce = "";
                                let mut salt_b64 = "";
                                let mut iter: u32 = 0;
                                for part in server_first.split(',') {
                                    if let Some(rest) = part.strip_prefix("r=") { server_nonce = rest; }
                                    else if let Some(rest) = part.strip_prefix("s=") { salt_b64 = rest; }
                                    else if let Some(rest) = part.strip_prefix("i=") { iter = rest.parse::<u32>().map_err(|e| DbError::ProtocolError(e.to_string()))?; }
                                }
                                let salt = general_purpose::STANDARD.decode(salt_b64).map_err(|e| DbError::ProtocolError(e.to_string()))?;
                                let pw = self.password.as_ref().unwrap();
                                // Derive salted password using ring
                                let mut salted_password = [0u8; 32];
                                pbkdf2::derive(pbkdf2::PBKDF2_HMAC_SHA256, NonZeroU32::new(iter).unwrap(), &salt, pw.as_bytes(), &mut salted_password);
                                // Client key & Stored key
                                let client_key_raw = hmac::sign(&hmac::Key::new(hmac::HMAC_SHA256, &salted_password), b"Client Key");
                                let client_key = client_key_raw.as_ref().to_vec();
                                let stored_key = digest::digest(&digest::SHA256, &client_key);
                                // Store serverKey for final verification
                                let server_key_raw = hmac::sign(&hmac::Key::new(hmac::HMAC_SHA256, &salted_password), b"Server Key");
                                scram_server_key = Some(server_key_raw.as_ref().to_vec());
                                // Client-final-message without proof
                                let gs2 = "n,,";
                                let channel_binding = general_purpose::STANDARD.encode(gs2.as_bytes());
                                let client_final_without_proof = format!("c={},r={}", channel_binding, server_nonce);
                                // Auth message
                                let auth_msg = format!("{},{},{}", client_first_bare, server_first, client_final_without_proof);
                                // Save auth message for signature verification
                                scram_auth_msg = Some(auth_msg.clone());
                                // Client signature using ring
                                let client_signature_raw = hmac::sign(&hmac::Key::new(hmac::HMAC_SHA256, stored_key.as_ref()), auth_msg.as_bytes());
                                let client_signature = client_signature_raw.as_ref();
                                // Client proof
                                let client_proof: Vec<u8> = client_key.iter().zip(client_signature.iter()).map(|(&x, &y)| x ^ y).collect();
                                let proof_b64 = general_purpose::STANDARD.encode(&client_proof);
                                let client_final_msg = format!("{},p={}", client_final_without_proof, proof_b64);
                                // Send SASLResponse
                                let mut msg3 = Vec::new();
                                msg3.push(b'p');
                                let len3 = (client_final_msg.len() + 4) as u32;
                                msg3.extend_from_slice(&len3.to_be_bytes());
                                msg3.extend_from_slice(client_final_msg.as_bytes());
                                conn.write_all(&msg3).await.map_err(|e| DbError::ProtocolError(e.to_string()))?;
                                conn.flush().await.map_err(|e| DbError::ProtocolError(e.to_string()))?;
                                continue;
                            } else {
                                return Err(DbError::ProtocolError("SCRAM client state missing".to_string()));
                            }
                        } else if code == 12 {
                            // AuthenticationSASLFinal (SCRAM server-final)
                            let server_final = String::from_utf8(payload[4..].to_vec()).map_err(|e| DbError::ProtocolError(e.to_string()))?;
                            if let Some(err_msg) = server_final.strip_prefix("e=") {
                                return Err(DbError::ProtocolError(format!("SCRAM error: {}", err_msg)));
                            }
                            // Expect v=<serverSignature>
                            let server_sig_b64 = server_final.strip_prefix("v=").ok_or_else(|| DbError::ProtocolError("Missing server signature".to_string()))?;
                            let server_sig = general_purpose::STANDARD.decode(server_sig_b64).map_err(|e| DbError::ProtocolError(e.to_string()))?;
                            // Verify server signature
                            let server_key = scram_server_key.take().ok_or_else(|| DbError::ProtocolError("SCRAM server key missing".to_string()))?;
                            let auth_msg = scram_auth_msg.take().ok_or_else(|| DbError::ProtocolError("SCRAM auth message missing".to_string()))?;
                            let expected_sig_raw = hmac::sign(&hmac::Key::new(hmac::HMAC_SHA256, &server_key), auth_msg.as_bytes());
                            if expected_sig_raw.as_ref() != server_sig.as_slice() {
                                return Err(DbError::ProtocolError("SCRAM server signature mismatch".to_string()));
                            }
                            continue;
                        } else {
                            return Err(DbError::ProtocolError(format!("Unsupported authentication code {}", code)));
                        }
                    }
                    b'E' => {
                        // ErrorResponse
                        let msg = String::from_utf8_lossy(&payload[..payload.len()-1]).to_string();
                        return Err(DbError::QueryError(msg));
                    }
                    b'Z' => {
                        // ReadyForQuery
                        break;
                    }
                    _ => {
                        // Ignore other messages (ParameterStatus, BackendKeyData, etc.)
                    }
                }
            }
        }
        // Return connection with handshake completed
        Ok(DbConnection {
            host: self.host.clone(),
            port: self.port,
            protocol: self.protocol.clone(),
            database: self.database.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
            stream: Some(conn),
        })
    }
}

/// Represents an active database connection with PostgreSQL specifics.
pub struct DbConnection {
    host: String,
    port: u16,
    protocol: Protocol,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
    pub(super) stream: Option<GenericConnection>,  // Expose stream to sql module for query access
}

impl DbConnection {
    /// Closes the database connection.
    pub async fn close(&mut self) -> Result<(), DbError> {
        if let Some(mut conn) = self.stream.take() {
            use tokio::io::AsyncWriteExt;
            conn.shutdown().await.map_err(|e| DbError::ConnectionError(e.to_string()))?;
        }
        Ok(())
    }

    // Additional methods for database operations will be added in query.rs
}

#[async_trait]  
impl Tx for DbConnection {
    type Request = ();
    type Response = DbConnection;
    type Config = DbConnectionBuilder;
    type Error = DbError;

    async fn process(&mut self, _: Self::Request) -> Result<&mut Self::Response, Self::Error> {
        Ok(self)
    }

    async fn shutdown(&mut self) -> Result<(), Self::Error> {
        self.close().await
    }

    async fn fetch<T: Into<String> + Send + Sync>(_: T, _: Self::Request, config: Self::Config) -> Result<Self::Response, Self::Error> {
        config.connect().await 
    }
}
