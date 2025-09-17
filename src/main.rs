use crate::config::{Config, ExitStatus};

mod common;
mod config;
mod frontend;

fn main() -> ExitStatus
{
    Config::from_args()
        .map(|x| x.compile())
        .map_err(|x| ExitStatus::from(x))
        .unwrap_or_else(|x| x)
}
