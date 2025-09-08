use std::fmt::Debug;

use crate::config::{Config, Status};

mod common;
mod config;
mod frontend;

fn main() -> Status
{
    Config::from_args()
        .map(|x| x.compile())
        .map_err(|x| Status::Config(x))
        .unwrap_or_else(|x| x)
}
