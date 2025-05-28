use starberry::prelude::*; 

pub static APP: SApp = Lazy::new(|| {
    App::new().build() 
}); 
