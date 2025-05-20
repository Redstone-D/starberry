use std::fmt::Write;

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};

use super::meta::{HttpMeta, ParseConfig}; 
use super::body::HttpBody; 

pub async fn parse_lazy<R: AsyncRead + Unpin>(stream: &mut BufReader<R>, config: &ParseConfig, is_request: bool, print_raw: bool) -> std::io::Result<(HttpMeta, HttpBody)> {
    // Create one BufReader up-front, pass this throughout.
    let meta = HttpMeta::from_stream(
        stream, 
        config, 
        print_raw, 
        is_request 
    ).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?; 

    let body = HttpBody::Unparsed; 

    Ok((meta, body)) 
} 

pub async fn parse_body<R: AsyncRead + Unpin>(meta: &mut HttpMeta, body: &mut HttpBody, reader: &mut BufReader<R>, max_size: usize) -> std::io::Result<()> {
    if let HttpBody::Unparsed = *body {
        *body = HttpBody::parse(
            reader,
            max_size, 
            meta,
        ).await;
    }
    Ok(())
} 

pub async fn send<W: AsyncWrite +  Unpin>(meta: &mut HttpMeta, body: &mut HttpBody, stream: &mut BufWriter<W>) -> std::io::Result<()> {
    let mut writer = BufWriter::new(stream);
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

    println!("{:?}, {:?}", headers, bin); 
    writer.flush().await?; 
    
    Ok(()) 
} 
