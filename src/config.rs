use std::{
    env,
    fmt::{Debug, Display},
    process::{ExitCode, Termination},
};

use crate::frontend::frontend;

pub struct Config
{
    filename: String,
}

pub enum ConfigError
{
    NoFileProvided,
    IoError,
}

impl Debug for ConfigError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(
            f,
            "{}",
            match self
            {
                Self::NoFileProvided => "No input file provided",
                Self::IoError => "IO Error while reading file",
            }
        )
    }
}

pub enum Status
{
    Success,
    Config(ConfigError),
    Compile(Vec<String>),
}

impl Debug for Status
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::Success => write!(f, ""),
            Self::Config(e) => write!(f, "[Config Error] {:?}", e),
            Self::Compile(es) => write!(f, "{}", es.join("\n")),
        }
    }
}

impl Termination for Status
{
    fn report(self) -> std::process::ExitCode
    {
        match self
        {
            Status::Success => ExitCode::SUCCESS,
            e =>
            {
                eprintln!("{:?}", e);
                ExitCode::FAILURE
            }
        }
    }
}

impl Config
{
    pub fn from_args() -> Result<Self, ConfigError>
    {
        let mut args = env::args();

        // First argument (that isnt the executable name)
        let filename = args.skip(1).next().ok_or(ConfigError::NoFileProvided)?;

        Ok(Self { filename })
    }

    fn read_file(&self) -> Result<String, ConfigError>
    {
        std::fs::read_to_string(&self.filename).map_err(|_| ConfigError::IoError)
    }

    fn compile_helper(&self) -> Result<(), Status>
    {
        let prog = frontend(&self.read_file().map_err(|x| Status::Config(x))?)
            .map_err(|es| Status::Compile(es.into_iter().map(|x| x.format()).collect()))?;

        Ok(())
    }

    pub fn compile(&self) -> Status
    {
        self.compile_helper()
            .err()
            .unwrap_or_else(|| Status::Success)
    }
}
