use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream; 

use crate::connection::error::Result;

pub enum Connection {
    Tcp(TcpStream),
    Tls(tokio_rustls::client::TlsStream<TcpStream>),
}

impl Connection {
    /// Write data to the connection
    pub async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        match self {
            Connection::Tcp(stream) => stream.write_all(buf).await?,
            Connection::Tls(stream) => stream.write_all(buf).await?,
        }
        Ok(())
    }

    /// Read data from the connection
    pub async fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let result = match self {
            Connection::Tcp(stream) => stream.read_to_end(buf).await?,
            Connection::Tls(stream) => stream.read_to_end(buf).await?,
        };
        Ok(result)
    }
    
    /// Read a fixed number of bytes
    pub async fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        match self {
            Connection::Tcp(stream) => stream.read_exact(buf).await?,
            Connection::Tls(stream) => stream.read_exact(buf).await?,
        }; 
        Ok(())
    } 
}

impl AsyncRead for Connection {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Connection::Tcp(stream) => Pin::new(stream).poll_read(cx, buf),
            Connection::Tls(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for Connection {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            Connection::Tcp(stream) => Pin::new(stream).poll_write(cx, buf),
            Connection::Tls(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Connection::Tcp(stream) => Pin::new(stream).poll_flush(cx),
            Connection::Tls(stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Connection::Tcp(stream) => Pin::new(stream).poll_shutdown(cx),
            Connection::Tls(stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
} 

