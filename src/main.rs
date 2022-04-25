use std::time::Instant;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

mod cuckoofilter;
use cuckoofilter::CuckooFilter;

fn main() {
    let mut cf = CuckooFilter::new(4, 25);
    let mut cnt: usize = 0;
    let mut fp: usize = 0;
    let mut last_measured: Instant;
    if let Ok(lines) = read_lines("./uuids.txt") {
        last_measured = Instant::now();
        for line in lines {
            if let Ok(line) = line {
                if cf.contains(&line) {
                    fp += 1;
                }
                match cf.insert(&line) {
                    true => (),
                    false => {
                        println!("WARN: failed to insert {}", line);
                        println!("{:?}", cf.load_factor());
                    }
                }
                cnt += 1;
                if (cnt % 1000000) == 0 {
                    println!("N = {}, LF = {}, FP = {}, elapsed = {}ms", cnt, cf.load_factor(), fp, last_measured.elapsed().as_millis());
                    last_measured = Instant::now();
                }

            }
        }
    }
    println!("LF = {:?}, FP = {}", cf.load_factor(), fp);
    if let Ok(lines) = read_lines("./uuids.txt") {
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
