//! MOD-13: Penambang CPU Zevrigo Coin (`zevrigo-miner`).

#![forbid(unsafe_code)]

use std::process::ExitCode;

use zevrigo_miner::{
    ATTEMPTS_PER_TEMPLATE, Command, Miner, MinerPow, RANDOMX_ATTEMPTS_PER_TEMPLATE, RandomxMode,
    Round, USAGE, parse_args,
};
use zevrigo_primitives::Network;

const BIN_NAME: &str = "zevrigo-miner";

fn main() -> ExitCode {
    let config = match parse_args(std::env::args().skip(1)) {
        Ok(Command::Help) => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Ok(Command::Version) => {
            println!("{BIN_NAME} {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Ok(Command::Run(config)) => config,
        Err(error) => {
            eprintln!("{BIN_NAME}: {error}");
            eprintln!();
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    let (pow, attempts) = match (config.network, config.light) {
        (Network::Regtest, _) => (MinerPow::Regtest, ATTEMPTS_PER_TEMPLATE),
        (_, true) => (
            MinerPow::Randomx(RandomxMode::Light),
            RANDOMX_ATTEMPTS_PER_TEMPLATE,
        ),
        (_, false) => {
            println!(
                "RandomX mode cepat: dataset sekitar 2 GiB disusun saat kunci baru, sekitar 1 menit"
            );
            (
                MinerPow::Randomx(RandomxMode::Fast),
                RANDOMX_ATTEMPTS_PER_TEMPLATE,
            )
        }
    };
    let result = Miner::connect_with_pow(
        &config.rpc_url,
        config.payout_kind,
        config.payout_payload,
        attempts,
        pow,
    )
    .and_then(|miner| miner.check_network(config.network).map(|()| miner))
    .and_then(|miner| {
        miner.run(config.blocks, |round| {
            if let Round::Accepted {
                height,
                block_id,
                nonce,
            } = round
            {
                let id = block_id
                    .as_ref()
                    .map(|id| String::from(*id))
                    .unwrap_or_default();
                println!("blok diterima: tinggi {height}, nonce {nonce}, id {id}");
            }
        })
    });
    match result {
        Ok(count) => {
            println!("selesai: {count} blok ditambang");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{BIN_NAME}: {error}");
            ExitCode::FAILURE
        }
    }
}
