use hyper;
use luminal_router::{LuminalError, LuminalErrorKind};

error_chain!{
    links {
        Luminal(LuminalError, LuminalErrorKind);
    }

    foreign_links {
        Hyper(hyper::Error);
    }
}
