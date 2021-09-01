use std::{
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    thread,
    time::Duration,
};

use anyhow::Error;
use regex::Regex;
use structopt::StructOpt;

use mznlaunch::{psp::load_psp, timeout::timeout, tsptw::load_tsptw};

#[derive(StructOpt)]
enum Args {
    Tsptw {
        fname: String,
        #[structopt(long, short, default_value = "60")]
        expiry: u64,
    },
    Psp {
        fname: String,
        #[structopt(long, short, default_value = "60")]
        expiry: u64,
    },
}

fn main() -> Result<(), Error> {
    let args = Args::from_args();
    //let tsptw= load_tsptw(&args.fname)?;
    //println!("{}", tsptw.to_minizinc(&name(&args.fname)));
    match args {
        Args::Tsptw { fname, expiry } => {
            let child = invoke_tsptw_mzn(&fname)?;
            timeout(child, Duration::from_secs(expiry))?;
        }
        Args::Psp { fname, expiry } => {
            let child = invoke_psp_mzn(&fname)?;
            timeout(child, Duration::from_secs(expiry))?;
        }
    }

    Ok(())
}

/// This function transforms the given instance into a format which is
/// understood by minizinc. Then it invokes minizinc to solve that instance.
/// It returns a hook to the underlying minizinc process.
pub fn invoke_psp_mzn(fname: &str) -> Result<Child, Error> {
    let psp = load_psp(fname)?;
    let iname = name(fname);
    let dzn = psp.to_minizinc();

    let mut child = Command::new("minizinc")
        .arg("--intermediate")
        .arg("--output-time")
        //.arg("--parallel")
        //.arg(num_cpus::get().to_string())
        .arg("--input-from-stdin")
        //.arg("psp.mzn")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let instance = include_str!("../psp.mzn");

    let mut stdin = child.stdin.take().expect("Failed to take stdin");
    stdin.write_all(dzn.as_bytes())?;
    stdin.write_all(instance.as_bytes())?;

    let stdout = child.stdout.take().expect("Failed to take stdout");
    let stdout = BufReader::new(stdout);
    spawn_psp_output_logger(iname, stdout);

    Ok(child)
}

/// This function transforms the given instance into a format which is
/// understood by minizinc. Then it invokes minizinc to solve that instance.
/// It returns a hook to the underlying minizinc process.
pub fn invoke_tsptw_mzn(fname: &str) -> Result<Child, Error> {
    let tsptw = load_tsptw(fname)?;
    let iname = name(fname);
    let dzn = tsptw.to_minizinc();

    let mut child = Command::new("minizinc")
        .arg("--intermediate")
        .arg("--output-time")
        //.arg("--parallel")
        //.arg(num_cpus::get().to_string())
        .arg("--input-from-stdin")
        //.arg("tsptw.mzn")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let instance = include_str!("../tsptw.mzn");

    let mut stdin = child.stdin.take().expect("Failed to take stdin");
    stdin.write_all(dzn.as_bytes())?;
    stdin.write_all(instance.as_bytes())?;

    let stdout = child.stdout.take().expect("Failed to take stdout");
    let stdout = BufReader::new(stdout);
    spawn_tsptw_output_logger(iname, stdout);

    Ok(child)
}

/// Spawns a thread which processes the minizinc output and formats is nicely
fn spawn_tsptw_output_logger<T: 'static + Send + Read>(iname: String, stdout: BufReader<T>) {
    thread::spawn(move || {
        let mut makespan: f32 = f32::MAX;
        let mut permutation: String = "-- no solution --".to_string();
        let mut elapsed: f32 = 0_f32;

        let re_makespan =
            Regex::new(r"^% makespan: (\d+.\d+)").expect("failed to compile makespan pattern");
        let re_permutation =
            Regex::new(r"^% permutation: \[(.*)\]").expect("failed to compile permutation pattern");
        let re_elapsed =
            Regex::new(r"^% time elapsed: (\d+.\d+) s").expect("failed to compile elapsed pattern");

        for line in stdout.lines() {
            let line = line.unwrap();

            if let Some(cap) = re_makespan.captures(&line) {
                makespan = (&cap[1]).parse::<f32>().unwrap();
            }
            if let Some(cap) = re_permutation.captures(&line) {
                permutation = cap[1].replace(",", "").to_string()
            }
            if let Some(cap) = re_elapsed.captures(&line) {
                elapsed = cap[1].parse::<f32>().unwrap();
            }

            if line == "----------" {
                println!(
                    "{:<10} | {:>10.4} | {:>10.2} | {}",
                    iname, makespan, elapsed, permutation
                );
            }
        }
    });
}

/// Spawns a thread which processes the minizinc output and formats is nicely
fn spawn_psp_output_logger<T: 'static + Send + Read>(iname: String, stdout: BufReader<T>) {
    thread::spawn(move || {
        let mut total_cost: f32 = f32::MAX;
        let mut plan: String = "-- no solution --".to_string();
        let mut elapsed: f32 = 0_f32;

        let re_total_cost =
            Regex::new(r"^% total cost : (\d+.\d+)").expect("failed to compile total cost pattern");
        let re_plan =
            Regex::new(r"^% plan       : \[(.*)\]").expect("failed to compile plan pattern");
        let re_elapsed =
            Regex::new(r"^% time elapsed: (\d+.\d+) s").expect("failed to compile elapsed pattern");

        for line in stdout.lines() {
            let line = line.unwrap();

            if let Some(cap) = re_total_cost.captures(&line) {
                total_cost = (&cap[1]).parse::<f32>().unwrap();
            }
            if let Some(cap) = re_plan.captures(&line) {
                plan = cap[1].replace(",", "").to_string()
            }
            if let Some(cap) = re_elapsed.captures(&line) {
                elapsed = cap[1].parse::<f32>().unwrap();
            }

            if line == "----------" {
                println!(
                    "{:<10} | {:>10.4} | {:>10.2} | {}",
                    iname, total_cost, elapsed, plan
                );
            }
        }
    });
}

pub fn name(fname: &str) -> String {
    let path = PathBuf::from(fname);
    let bench = path
        .parent()
        .map(|x| x.file_name().unwrap().to_str().unwrap())
        .unwrap_or_default();
    let name = path
        .file_name()
        .map(|x| x.to_str().unwrap())
        .unwrap_or_default();
    format!("{}/{}", bench, name)
}
