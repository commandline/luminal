#[macro_use]
extern crate error_chain;

mod error;
mod types;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
