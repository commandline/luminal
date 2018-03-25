extern crate error_chain;
extern crate futures;
extern crate hyper;
extern crate luminal_example;
extern crate luminal_handler;
extern crate luminal_router;

use error_chain::ChainedError;

pub fn main() {
    if let Err(error) = luminal_example::run() {
        println!("{}", error.display_chain());
    }
}
