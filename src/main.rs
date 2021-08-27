mod tsptw;
mod utils;

use std::{
    io::{BufRead, BufReader, Read, Write},
    ops::DerefMut,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};

use anyhow::Error;
use killall::{kill, list_descendants};
use regex::Regex;
use structopt::StructOpt;
use tsptw::load_tsptw;

#[derive(StructOpt)]
struct Args {
    fname: String,
    #[structopt(long, short, default_value = "60")]
    timeout: u64,
}

fn main() -> Result<(), Error> {
    let args = Args::from_args();
    //let tsptw= load_tsptw(&args.fname)?;
    //println!("{}", tsptw.to_minizinc(&name(&args.fname)));
    let child = invoke_mzn(&args.fname)?;

    timeout(child, Duration::from_secs(args.timeout))?;

    Ok(())
}

/// This function transforms the given instance into a format which is
/// understood by minizinc. Then it invokes minizinc to solve that instance.
/// It returns a hook to the underlying minizinc process.
pub fn invoke_mzn(fname: &str) -> Result<Child, Error> {
    let tsptw = load_tsptw(fname)?;
    let iname = name(fname);
    let dzn = tsptw.to_minizinc();

    let mut child = Command::new("minizinc")
        .arg("--intermediate")
        .arg("--output-time")
        .arg("--parallel")
        .arg(num_cpus::get().to_string())
        .arg("--input-from-stdin")
        .arg("tsptw.mzn")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().expect("Failed to take stdin");
    stdin.write_all(dzn.as_bytes())?;

    let stdout = child.stdout.take().expect("Failed to take stdout");
    let stdout = BufReader::new(stdout);
    spawn_output_logger(iname, stdout);

    Ok(child)
}

fn timeout(child: Child, timeout: Duration) -> Result<(), Error> {
    let flag_cond = Arc::new((Mutex::new((child, false)), Condvar::new()));

    let flag_cond2 = Arc::clone(&flag_cond);
    thread::spawn(move || loop {
        {
            let (lock, cond) = flag_cond2.as_ref();
            let mut lock = lock.lock().expect("lock");
            let (ref mut child, ref mut done) = *lock;
            if *done {
                break;
            }
            if let Ok(Some(_status)) = child.try_wait() {
                *done = true;
                cond.notify_all();
            }
        }
        thread::sleep(Duration::from_millis(500));
    });

    let (ref flag, ref cond) = flag_cond.as_ref();
    let lock = flag.lock().expect("poisoned");
    let (mut guard, _) = cond
        .wait_timeout_while(lock, timeout, |&mut (_, complete)| !complete)
        .expect("wait_timeout_while");

    let (child, done) = guard.deref_mut();
    *done = true;
    match child.try_wait()? {
        Some(_status) => {}
        None => {
            let childrens =
                list_descendants(child.id() as usize).expect("cannot list children to kill");
            for kid in childrens {
                kill(&kid).expect("child would not die");
            }
        }
    }

    Ok(())
}

/// Spawns a thread which processes the minizinc output and formats is nicely
fn spawn_output_logger<T: 'static + Send + Read>(iname: String, stdout: BufReader<T>) {
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
