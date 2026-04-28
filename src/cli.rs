//! CLI argument parsing and the headless `--dump` mode.

use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;

use crate::map::gen::{bsp_generate, BspConfig};

pub struct CliOpts {
    pub seed: Option<u64>,
    pub dump: bool,
    pub dump_count: u32,
    pub dump_width: i32,
    pub dump_height: i32,
}

pub fn parse_args() -> Result<CliOpts> {
    let mut opts = CliOpts {
        seed: None,
        dump: false,
        dump_count: 1,
        dump_width: 60,
        dump_height: 24,
    };
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => {
                let raw = args
                    .next()
                    .ok_or_else(|| anyhow!("--seed requires a value"))?;
                opts.seed = Some(parse_seed_value(&raw)?);
            }
            other if other.starts_with("--seed=") => {
                opts.seed = Some(parse_seed_value(&other[7..])?);
            }
            "--dump" => opts.dump = true,
            "--count" => {
                let raw = args.next().ok_or_else(|| anyhow!("--count requires a value"))?;
                opts.dump_count = raw.parse().context("parse --count")?;
            }
            "--width" => {
                let raw = args.next().ok_or_else(|| anyhow!("--width requires a value"))?;
                opts.dump_width = raw.parse().context("parse --width")?;
            }
            "--height" => {
                let raw = args.next().ok_or_else(|| anyhow!("--height requires a value"))?;
                opts.dump_height = raw.parse().context("parse --height")?;
            }
            "--help" | "-h" => {
                eprintln!(
                    "usage: crawl-rs [--seed N] [--dump [--count N] [--width N] [--height N]]"
                );
                std::process::exit(0);
            }
            other => return Err(anyhow!("unknown argument: {other}")),
        }
    }
    Ok(opts)
}

pub fn parse_seed_value(raw: &str) -> Result<u64> {
    if let Some(hex) = raw.strip_prefix("0x") {
        u64::from_str_radix(hex, 16).context("parse seed (hex)")
    } else {
        raw.parse::<u64>().context("parse seed (decimal)")
    }
}

pub fn seed_from_clock() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| {
            let nanos = d.as_nanos() as u64;
            nanos ^ (std::process::id() as u64).rotate_left(17)
        })
        .unwrap_or(0xdead_beef_cafe_babe)
}

pub fn dump_maps(
    seed: Option<u64>,
    count: u32,
    width: i32,
    height: i32,
) -> Result<()> {
    let base_seed = seed.unwrap_or_else(seed_from_clock);
    let cfg = BspConfig::default();
    for i in 0..count {
        let s = base_seed.wrapping_add(i as u64);
        let mut rng = Pcg64Mcg::seed_from_u64(s);
        let d = bsp_generate(width, height, &cfg, &mut rng);
        println!("--- seed {:#018x} rooms={} ---", s, d.rooms.len());
        for y in 0..d.map.height() {
            let mut line = String::with_capacity(d.map.width() as usize);
            for x in 0..d.map.width() {
                let ch = match d.map.tile(x, y) {
                    Some(t) => t.glyph(),
                    None => ' ',
                };
                line.push(ch);
            }
            println!("{line}");
        }
        println!();
    }
    Ok(())
}
