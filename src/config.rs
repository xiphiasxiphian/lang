use std::{
    env::{self, Args}, fmt::Debug, process::{ExitCode, Termination}
};

use crate::{common::ScopeMethods, frontend::frontend};

pub struct Config
{
    filename: String, // Required
    error_code: ExitCode, // Default
}

impl Default for Config
{
    fn default() -> Self {
        Self {
            filename: Default::default(),
            error_code: ExitCode::FAILURE
        }
    }
}

pub enum ConfigError
{
    NoFileProvided,
    IoError,
    UnknownFlag(String),
    InvalidArgument(String),
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
                Self::NoFileProvided => "No input file provided".into(),
                Self::IoError => "IO Error while reading file".into(),
                Self::UnknownFlag(flag) => format!("Unknown flag {flag}"),
                Self::InvalidArgument(flag) => format!("Invalid argument provided for {flag}"),
            }
        )
    }
}

pub struct ExitStatus(ExitCode, Status);
impl ExitStatus
{
    pub const SUCCESS: Self = Self(ExitCode::SUCCESS, Status::Success);

    fn from_raw_status(value: Status, compile_code: ExitCode) -> Self
    {
        match value
        {
            Status::Success => Self::SUCCESS,
            Status::Config(a) => a.into(),
            a @ Status::Compile(_) => Self(compile_code, a)
        }
    }
}

impl From<ConfigError> for ExitStatus
{
    fn from(value: ConfigError) -> Self {
        Self(Config::CONFIG_ERROR, Status::Config(value))
    }
}

enum Status
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

impl Termination for ExitStatus
{
    fn report(self) -> std::process::ExitCode
    {
        match self
        {
            Self(ex, Status::Success) => ex,
            Self(ex, e) =>
            {
                eprintln!("{:?}", e);
                ex
            }
        }
    }
}

impl Config
{
    pub const CONFIG_ERROR: ExitCode = ExitCode::FAILURE;

    pub fn from_args() -> Result<Self, ConfigError>
    {
        let mut args = env::args().skip(1); // Ignore executable name itself
        let mut config = Config::default();

        let mut set_filename: bool = false;
        while let Some(flag) = args.next()
        {
            match flag.as_str()
            {
                a @ "--fail" => {
                    let operand = args.next().ok_or(ConfigError::InvalidArgument(a.into()))?;
                    config.error_code = operand.parse::<u8>()
                        .map_err(|_| ConfigError::InvalidArgument(a.into()))?
                        .into();
                }
                file => {
                    // Cannot have more than one unnamed argument
                    if set_filename
                    {
                        return Err(ConfigError::UnknownFlag(file.into()))
                    }
                    else
                    {
                        config.filename = file.into();
                        set_filename = true;
                    }
                }
            }
        }

        Ok(config)
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

    pub fn compile(&self) -> ExitStatus
    {
        self.compile_helper()
            .map_err(|x| ExitStatus::from_raw_status(x, self.error_code))
            .err()
            .unwrap_or_else(|| ExitStatus::SUCCESS)
    }
}
