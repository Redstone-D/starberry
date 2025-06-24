use std::fmt::Write;

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};

use crate::http::http_value::StatusCode;

use super::meta::HttpMeta; 
use super::body::HttpBody; 
use super::safety::HttpSafety; 

pub async fn parse_lazy<R: AsyncRead + Unpin>(stream: &mut BufReader<R>, config: &HttpSafety, is_request: bool, print_raw: bool) -> Result<(HttpMeta, HttpBody), StatusCode> {
    // Create one BufReader up-front, pass this throughout.
    let meta = HttpMeta::from_stream(
        stream, 
        config, 
        print_raw, 
        is_request 
    ).await?; 

    let body = HttpBody::Unparsed; 

    Ok((meta, body)) 
} 

pub async fn parse_body<R: AsyncRead + Unpin>(meta: &mut HttpMeta, body: &mut HttpBody, reader: &mut BufReader<R>, safety_setting: &HttpSafety) -> Result<(), StatusCode> {
    if let HttpBody::Unparsed = *body {
        *body = HttpBody::parse(
            reader,
            meta,
            safety_setting 
        ).await;
    }
    Ok(())
} 

pub async fn send<W: AsyncWrite +  Unpin>(meta: &mut HttpMeta, body: &mut HttpBody, writer: &mut BufWriter<W>) -> std::io::Result<()> {
    let mut headers = String::with_capacity(256); 

    // Add the values such as content length into header 
    let bin = body.into_static(meta).await; 
    write!( 
        &mut headers,
        "{}", 
        meta.represent()
    ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    writer.write_all(headers.as_bytes()).await?;
    writer.write_all(bin).await?; 

    // println!("{:?}, {:?}", headers, bin); 
    writer.flush().await?; 
    
    Ok(()) 
} 
