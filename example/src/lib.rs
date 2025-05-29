use starberry::prelude::*; 

pub static APP: SApp = Lazy::new(|| {
    App::new().build() 
}); 

#[url(reg![&APP, LitUrl("")])] 
pub async fn index() -> HttpResponse {
    text_response("Hello 0.6!") 
} 
