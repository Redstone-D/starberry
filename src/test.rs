use starberry::app::app::App;
use starberry::app::urls;
use starberry::http::http_value::*;
use starberry::http::response::*;
use std::sync::Arc;
use std::net::TcpListener;
use starberry::app::app::RunMode;
use std::time::Duration; 
use std::thread::sleep; 

pub async fn test() {
    let app = Arc::new(App {
        root_url: init_urls(),
        listener: TcpListener::bind("127.0.0.1:3003").unwrap(),
        mode: RunMode::Development,
    });
    app.run().await;
}

pub fn init_urls() -> urls::Url {
    urls::Url {
        path: urls::PathPattern::Literal("/".to_string()),
        children: urls::Children::Some(vec![
            urls::Url {
                path: urls::PathPattern::Literal("about".to_string()),
                children: urls::Children::Nil,
                method: Some(Box::new(|_req| async {
                    HttpResponse::new(
                        HttpVersion::Http11,
                        StatusCode::OK,
                        String::from("About Page"),
                    )
                })),
            },
            urls::Url {
                path: urls::PathPattern::Regex("[0-9]+".to_string()),
                children: urls::Children::Nil,
                method: Some(Box::new(|_req| async {
                    HttpResponse::new(
                        HttpVersion::Http11,
                        StatusCode::OK,
                        String::from("Number page"),
                    )
                })),
            },
            urls::Url {
                path: urls::PathPattern::Any,
                children: urls::Children::Nil,
                method: Some(Box::new(|_req| async {
                    sleep(Duration::from_secs(1));
                    println!("1");
                    sleep(Duration::from_secs(1));
                    println!("2");
                    sleep(Duration::from_secs(1));
                    println!("3");
                    HttpResponse::new(
                        HttpVersion::Http11,
                        StatusCode::OK,
                        String::from("Async Test Page"),
                    )
                })),
            },
        ]),
        method: Some(Box::new(|_req| async {
            HttpResponse::new(
                HttpVersion::Http11,
                StatusCode::OK,
                String::from("Home Page"),
            )
        })), 
    }  
}
