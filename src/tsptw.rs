use std::{f32, fs::File, io::{BufRead, BufReader, Lines, Read}, path::Path};

use anyhow::Error;

use crate::utils::Matrix;

/// This structure represents the TSP with time window instane.
#[derive(Clone)]
pub struct TSPTW {
    /// The number of nodes (including depot)
    pub nb_nodes   : usize, 
    /// This is the distance matrix between any two nodes
    pub distances  : Matrix<usize>,
    /// This vector encodes the time windows to reach any vertex
    pub timewindows: Vec<TimeWindow>,
}

impl TSPTW {
    pub fn to_minizinc(&self, instance_name: &str) -> String {
        format!("
instance_name = \"{}\";
n = {};
distance = {};
time_window = {};", 
        instance_name,
        self.nb_nodes,
        self.dist_matrix(), 
        self.tw_matrix())
    }
    fn dist_matrix(&self) -> String {
        let mut out = "[|".to_string();
        for row in 0..self.distances.rows() {
            let line = self.distances.row(row)
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
    fn tw_matrix(&self) -> String {
        let body = self.timewindows.iter()
            .map(|tw| format!("{}, {}", tw.earliest, tw.latest))
            .collect::<Vec<String>>()
            .join("|");
        format!("[| {} |]", body)
    }
}

//-----------------------------------------------------------------------------
//--- UTILITIES ---------------------------------------------------------------
//-----------------------------------------------------------------------------

/// This represents the postition of the salesman in his tour.
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct Position(u16);

/// This structure, represents a timewindow. Basically it is nothing but a 
/// closed time interval
#[derive(Debug, Copy, Clone)]
pub struct TimeWindow {
    pub earliest: usize,
    pub latest  : usize
}

//-----------------------------------------------------------------------------
//--- PARSING -----------------------------------------------------------------
//-----------------------------------------------------------------------------
pub fn load_tsptw<P: AsRef<Path>>(path: P) -> Result<TSPTW, Error> {
    Ok(TSPTW::from(File::open(path)?))
}

impl From<File> for TSPTW {
    fn from(file: File) -> Self {
        Self::from(BufReader::new(file))
    }
}
impl <S: Read> From<BufReader<S>> for TSPTW {
    fn from(buf: BufReader<S>) -> Self {
        Self::from(buf.lines())
    }
}
impl <B: BufRead> From<Lines<B>> for TSPTW {
    fn from(lines: Lines<B>) -> Self {
        let mut lc         = 0;
        let mut nb_nodes   = 0;
        let mut distances  = Matrix::new(nb_nodes as usize, nb_nodes as usize, 0);
        let mut timewindows= vec![];

        for line in lines {
            let line = line.unwrap();
            let line = line.trim();

            // skip comment lines
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            
           // First line is the number of nodes
           if lc == 0 { 
               nb_nodes  = line.split_whitespace().next().unwrap().to_string().parse::<usize>().unwrap();
               distances = Matrix::new(nb_nodes as usize, nb_nodes as usize, 0);
           }
           // The next 'nb_nodes' lines represent the distances matrix
           else if (1..=nb_nodes).contains(&lc) {
               let i = (lc - 1) as usize;
               for (j, distance) in line.split_whitespace().enumerate() {
                    let distance = distance.to_string().parse::<f32>().unwrap();
                    let distance = (distance * 10000.0) as usize;
                    distances[(i, j)] = distance;
               }
           }
           // Finally, the last 'nb_nodes' lines impose the time windows constraints
           else {
               let mut tokens = line.split_whitespace();
               let earliest   = tokens.next().unwrap().to_string().parse::<f32>().unwrap();
               let latest     = tokens.next().unwrap().to_string().parse::<f32>().unwrap();

               let earliest   = (earliest * 10000.0) as usize;
               let latest     = (latest   * 10000.0) as usize;

               let timewind   = TimeWindow{earliest, latest};
               timewindows.push(timewind);
           }
            
            lc += 1;
        }

        TSPTW{nb_nodes, distances, timewindows}
    }
}