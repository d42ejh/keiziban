mod constant;
pub mod middleware;
mod route;
mod utility;

pub mod routes {
    pub use crate::route::*;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
