use std::{
    fs::File,
    io::{BufRead, BufReader, Lines, Read},
    path::Path,
};

use crate::{errors::Error, matrix::Matrix};

#[derive(Debug, Clone)]
pub struct Psp {
    pub n_items: usize,
    pub horizon: usize,

    pub changeover: Matrix<usize>,
    pub stocking: Vec<usize>,

    pub demands: Vec<Vec<usize>>,
}

impl Psp {
    pub fn to_minizinc(&self) -> String {
        format!(
            "
n = {};
horizon = {};
changeover = {};
stocking = {:?};
demands = {};
",
            self.n_items,
            self.horizon,
            self.co_matrix(),
            self.stocking,
            self.dem_matrix(),
        )
    }
    fn co_matrix(&self) -> String {
        let mut out = "[|".to_string();
        for row in 0..self.changeover.rows() {
            let line = self
                .changeover
                .row(row)
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(",");

            if row > 0 {
                out.push_str(" | ");
            }
            out.push_str(&line);
        }
        out.push_str("|]");
        out
    }
    fn dem_matrix(&self) -> String {
        let mut out = "[|".to_string();
        let mut first = true;
        for row in self.demands.iter() {
            let line = row
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(",");

            if first {
                first = false;
            } else {
                out.push_str(" | ");
            }
            out.push_str(&line);
        }
        out.push_str("|]");
        out
    }
}

//-----------------------------------------------------------------------------
//--- PARSING -----------------------------------------------------------------
//-----------------------------------------------------------------------------
pub fn load_psp<P: AsRef<Path>>(path: P) -> Result<Psp, Error> {
    Ok(Psp::from(File::open(path)?))
}

impl From<File> for Psp {
    fn from(file: File) -> Psp {
        BufReader::new(file).into()
    }
}
impl<S: Read> From<BufReader<S>> for Psp {
    fn from(buf: BufReader<S>) -> Psp {
        buf.lines().into()
    }
}
impl<B: BufRead> From<Lines<B>> for Psp {
    fn from(mut lines: Lines<B>) -> Psp {
        let horizon = lines.next().unwrap().unwrap().parse::<usize>().unwrap(); // damn you Result !
        let n_items = lines.next().unwrap().unwrap().parse::<usize>().unwrap(); // damn you Result !
        let _nb_orders = lines.next().unwrap().unwrap().parse::<usize>().unwrap(); // damn you Result !

        let _blank = lines.next();
        let mut changeover = Matrix::new(n_items, n_items, 0);

        let mut i = 0;
        for line in &mut lines {
            let line = line.unwrap();
            let line = line.trim();
            if line.is_empty() {
                break;
            }

            let costs = line.split_whitespace();
            for (other, cost) in costs.enumerate() {
                changeover[(i, other)] = cost.parse::<usize>().unwrap();
            }

            i += 1;
        }

        let stocking = lines
            .next()
            .unwrap()
            .unwrap()
            .split_whitespace()
            .map(|x| x.parse::<usize>().unwrap())
            .collect::<Vec<usize>>();

        let _blank = lines.next();

        let mut demands = vec![vec![0; horizon]; n_items];
        i = 0;
        for line in &mut lines {
            let line = line.unwrap();
            let line = line.trim();

            if line.is_empty() {
                break;
            }

            let demands_for_item = line.split_whitespace().map(|n| n.parse::<usize>().unwrap());

            for (period, demand) in demands_for_item.enumerate() {
                demands[i][period] += demand;
            }

            i += 1;
        }

        Psp {
            n_items,
            horizon,

            changeover,
            stocking,
            demands,
        }
    }
}
