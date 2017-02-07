extern crate uuid;
extern crate rustc_serialize;

pub mod request;
pub use request::Request;
pub use request::RequestType;
pub use request::REQUEST_TYPE_VERB_MAP;
