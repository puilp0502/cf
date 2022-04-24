use std::time::{Duration, Instant};
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

mod cuckoofilter;
use cuckoofilter::CuckooFilter;

fn main() {
    let mut cf = CuckooFilter::new(4, 24);
    let mut cnt: usize = 0;
    let mut last_measured: Instant;
    if let Ok(lines) = read_lines("./testset/uuids.txt") {
        last_measured = Instant::now();
        for line in lines {
            if let Ok(line) = line {
                match cf.insert(&line) {
                    true => (),
                    false => {
                        println!("WARN: failed to insert {}", line);
                        println!("{:?}", cf.load_factor());
                    }
                }
                cnt += 1;
                if (cnt % 1000000) == 0 {
                    println!("N = {}, LF = {}, elapsed = {}ms", cnt, cf.load_factor(), last_measured.elapsed().as_millis());
                    last_measured = Instant::now();
                }

            }
        }
    }
    println!("{:?}", cf.load_factor());
    if let Ok(lines) = read_lines("./testset/uuids.txt") {
        for line in lines {
            if let Ok(line) = line {
                match cf.contains(&line) {
                    true => (),
                    false => {
                        println!("WARN: failed to find {}", line);
                    }
                }
            }
        }
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
