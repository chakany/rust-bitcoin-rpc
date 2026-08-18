//! A rough subset of `bitcoin-cli`, to show the async client in use.
//!
//! ```text
//! cargo run --features aio --example btc-cli -- getblockchaininfo
//! cargo run --features aio --example btc-cli -- --rpcuser u --rpcpassword p getnetworkinfo
//! ```

use std::process::ExitCode;

use bitcoin_rpc::Auth;
use bitcoin_rpc::aio::ClientBuilder;
use bitcoin_rpc::prelude::aio::*;

const USAGE: &str = "\
usage: btc-cli [options] <command>

commands:
  getblockchaininfo
  getnetworkinfo
  getblockcount
  getbestblockhash

options:
  --rpcconnect <host>    node host (default 127.0.0.1)
  --rpcport <port>       node port (default 8332)
  --rpcuser <user>       username for basic auth
  --rpcpassword <pass>   password for basic auth
  --rpccookiefile <path> read credentials from Bitcoin Core's .cookie file
  -h, --help             show this help

If --rpcpassword is given, it takes precedence over --rpccookiefile, matching
bitcoin-cli (an omitted --rpcuser defaults to an empty username).
";

struct Options {
    host: String,
    port: u16,
    user: Option<String>,
    password: Option<String>,
    cookie_file: Option<String>,
    command: String,
}

fn parse_args() -> Result<Options, String> {
    let mut host = "127.0.0.1".to_string();
    let mut port = 8332u16;
    let mut user = None;
    let mut password = None;
    let mut cookie_file = None;
    let mut command = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = |name: &str| args.next().ok_or_else(|| format!("{name} needs a value"));
        match arg.as_str() {
            "-h" | "--help" => return Err(USAGE.to_string()),
            "--rpcconnect" => host = value("--rpcconnect")?,
            "--rpcport" => {
                port = value("--rpcport")?
                    .parse()
                    .map_err(|_| "--rpcport must be a number".to_string())?
            }
            "--rpcuser" => user = Some(value("--rpcuser")?),
            "--rpcpassword" => password = Some(value("--rpcpassword")?),
            "--rpccookiefile" => cookie_file = Some(value("--rpccookiefile")?),
            other if other.starts_with('-') => return Err(format!("unknown option {other}")),
            other => command = Some(other.to_string()),
        }
    }

    Ok(Options {
        host,
        port,
        user,
        password,
        cookie_file,
        command: command.ok_or_else(|| USAGE.to_string())?,
    })
}

#[tokio::main]
async fn main() -> ExitCode {
    let opts = match parse_args() {
        Ok(o) => o,
        Err(msg) => {
            eprintln!("{msg}");
            return ExitCode::FAILURE;
        }
    };

    // bitcoin-cli decides on the password alone, defaulting to an empty
    // username, and only falls back to the cookie file when no password was
    // given (see the caller of GetAuthCookie in bitcoin-cli.cpp).
    let auth = if let Some(password) = &opts.password {
        let user = opts.user.clone().unwrap_or_default();
        Auth::user_pass(user, password.as_str())
    } else if let Some(path) = &opts.cookie_file {
        Auth::cookie_file(path)
    } else {
        Auth::None
    };

    let client = match ClientBuilder::new(format!("http://{}:{}", opts.host, opts.port))
        .auth(auth)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let output = match opts.command.as_str() {
        "getblockchaininfo" => client
            .get_blockchain_info()
            .await
            .map(|v| format!("{v:#?}")),
        "getnetworkinfo" => client.get_network_info().await.map(|v| format!("{v:#?}")),
        "getblockcount" => client.get_block_count().await.map(|v| v.to_string()),
        "getbestblockhash" => client.get_best_block_hash().await,
        other => {
            eprintln!("unknown command `{other}`\n\n{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    match output {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
