pub mod board;
pub mod log;
mod login;
mod logout;
pub mod redirect;
mod register;
mod root;
pub mod rules;
mod search;
pub mod theme;
pub mod thread;
pub mod threadpost;
pub mod user;
pub use login::*;
pub use logout::*;
pub use register::*;
pub use root::*;
pub use search::*;
pub mod manage;
pub mod system_info;