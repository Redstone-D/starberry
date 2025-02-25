use starberry::app::app::App; 
use starberry::app::urls; 
use starberry::http::http_value::*; 
use starberry::http::response::*; 
use std::sync::Arc; 
use std::net::TcpListener; 
use starberry::app::app::RunMode; 

pub async fn test() {
    let app = Arc::new(App {
        root_url: init_urls(),
        listener: TcpListener::bind("127.0.0.1:3003").unwrap(),
        mode: RunMode::Development, 
    });
    app.run().await; 
}

pub fn init_urls() -> urls::Url{ 
    urls::Url {
        path: "/".to_string(),
        children: urls::Children::Some(vec![
            urls::Url {
                path: "about".to_string(),
                children: urls::Children::Nil,
                method: Some(Box::new(|req| async {
                    HttpResponse::new(HttpVersion::Http11, StatusCode::OK, String::from("About Page"))
                })),
            },
            urls::Url {
                path: "[0-9]+".to_string(),
                children: urls::Children::Nil,
                method: Some(Box::new(|req| async {
                    HttpResponse::new(HttpVersion::Http11, StatusCode::OK, String::from(""))
                })),
            },
        ]),
        method: Some(Box::new(|req| async {
            HttpResponse::new(HttpVersion::Http11, StatusCode::OK, String::from("Home Page"))
        })),
    } 
}