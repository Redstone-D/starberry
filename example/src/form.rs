use starberry::prelude::*;   

pub use crate::APP; 

static TEST_URL: SPattern = Lazy::new(|| {LitUrl("form")}); 


#[url(reg![&APP, TEST_URL, LitUrl("url_coded")])]  
async fn test_form() -> HttpResponse { 
    println!("Request to this dir"); 
    if req.method() == POST { 
        match req.form().await { 
            Some(form) => { 
                return text_response(format!("Form data: {:?}", form)); 
            } 
            None => { 
                return text_response("Error parsing form"); 
            }  
        } 
    } 
    plain_template_response("form.html") 
} 

#[url(APP.reg_from(&[TEST_URL.clone(), LitUrl("file")]))]  
async fn test_file() -> HttpResponse { 
    println!("Request to this dir"); 
    if req.method() == POST { 
        return text_response(format!("{:?}", req.files_or_default().await)); 
    } 
    akari_render!("form.html") 
} 

#[url(APP.reg_from(&[TEST_URL.clone(), LitUrl("cookie")]))]  
async fn test_cookie() -> HttpResponse { 
    if req.method() == POST { 
        match req.form().await { 
            Some(form) => { 
                let default_string = String::new(); 
                let name = form.get("name").unwrap_or(&default_string); 
                let value = form.get("value").unwrap_or(&default_string); 
                return text_response(format!("Cookie set data, {}: {}", name, value))
                    .add_cookie(name, Cookie::new(value)); 
            } 
            None => { 
                return text_response("Error parsing form"); 
            }  
        } 
    } 
    let cookies = req.get_cookies(); 
    // Convert cookies into a string in the same variable name 
    let mut scookie = String::new(); 
    for (name, value) in cookies.clone().into_iter() { 
        scookie.push_str(&format!("{}: {}\n", name, value)); 
    } 
    let scookie = object!(scookie); 
    akari_render!(
        "cookie.html", 
        current_cookie = scookie
    ) 
} 
