use std::str::FromStr;

use argh::FromArgs;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

mod protocol;
mod receiver;
mod sender;
mod utils;

#[derive(Debug, FromArgs)]
/// mirror system audio over network
struct Args {
    /// send or receive, default to send
    #[argh(positional, from_str_fn(Role::from_str))]
    role: Role,
    /// address to listen or connect
    #[argh(positional)]
    addr: String,
    /// buffer size, default to 128, will be clamp to supported range
    #[argh(option, default = "128")]
    buffer_size: u32,
}

#[derive(Debug, Clone, Copy)]
enum Role {
    Send,
    Recv,
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "send" => Self::Send,
            "recv" => Self::Recv,
            other => {
                return Err(format!("unknown role: {other}"));
            }
        })
    }
}

fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let args: Args = argh::from_env();
    match args.role {
        Role::Send => sender::run(&args),
        Role::Recv => receiver::run(&args),
    }
}
