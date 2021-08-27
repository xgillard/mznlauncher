mod utils;
mod tsptw;

use std::{io::Write, path::PathBuf, process::{Child, Command, Stdio}, thread, time::Duration};

use anyhow::Error;
use killall::{kill, list_descendants};
use structopt::StructOpt;
use tsptw::load_tsptw;

#[derive(StructOpt)]
struct Args {
    fname: String,
    #[structopt(long, short, default_value="60")]
    timeout: u64,
}

fn main() -> Result<(), Error> {
    let args = Args::from_args();
    //let tsptw= load_tsptw(&args.fname)?;
    //println!("{}", tsptw.to_minizinc(&name(&args.fname)));
    let mut child = invoke_mzn(&args.fname)?;
    thread::sleep(Duration::from_secs(args.timeout));

    match child.try_wait()? {
        Some(_status) => {}
        None => {
            let childrens = list_descendants(child.id() as usize).expect("cannot list children to kill");
            for kid in childrens {
                kill(&kid).expect("child would not die");
            }
        }
    }

    Ok(())
}

pub fn invoke_mzn(fname: &str) -> Result<Child, Error> {
    let tsptw = load_tsptw(fname)?;
    let iname = name(fname);
    let dzn   = tsptw.to_minizinc(&iname);

    let mut child = Command::new("minizinc")
        .arg("--intermediate")
        .arg("--output-time")
        .arg("--parallel").arg(num_cpus::get().to_string())
        .arg("--input-from-stdin")
        .arg("tsptw.mzn")
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .spawn()?;

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    stdin.write_all(dzn.as_bytes())?;

    Ok(child)
}

pub fn name(fname: &str) -> String {
    let path = PathBuf::from(fname);
    let bench= path.parent().map(|x|x.file_name().unwrap().to_str().unwrap()).unwrap_or_default();
    let name = path.file_name().map(|x|x.to_str().unwrap()).unwrap_or_default();
    format!("{}/{}", bench, name)
}