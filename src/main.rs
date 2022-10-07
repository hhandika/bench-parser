use std::io::prelude::*;
use std::{fs::File, io::BufReader, path::Path};

fn main() {
    // let path = "data/*.txt";
    // let mut files = glob::glob(path).unwrap();
    let test = Path::new("data/convert_bench_raw_aa_OpenSUSE_2022-10-05.txt");
    parse_text(test);
    // println!("{:?}", record);
}

fn parse_text(file_path: &Path) {
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);
    let records = BenchReader::new(reader);
    records.into_iter().for_each(|r| {
        println!();
        println!("CPU: {}", r.cpu);
        println!("OS: {}", r.os);
        println!("Dataset: {}", r.benchmark.bench);
        r.benchmark.dataset.iter().for_each(|d| {
            println!("Dataset: {}", d.name);
            println!("Bench: {:?}", d.result);
        });
        println!();
    });
}

struct BenchReader<R: Read> {
    reader: BufReader<R>,
    cpu: String,
    os: String,
    bench_name: String,
    dataset: Dataset,
    lcounts: usize,
}

impl<R: Read> BenchReader<R> {
    fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            cpu: String::new(),
            os: String::new(),
            bench_name: String::new(),
            dataset: Dataset::new(),
            lcounts: 0,
        }
    }

    fn next_record(&mut self) -> Option<Records> {
        while let Some(Ok(line)) = self.reader.by_ref().lines().next() {
            if line.starts_with("Model") {
                self.cpu = line.split(":").nth(1).unwrap().trim().to_string();
                self.os = String::from("openSUSE");
            } else if line.starts_with("Darwin") {
                self.cpu = String::from("Apple M1");
                self.os = String::from("macOS");
            } else if line.starts_with("Benchmarking") {
                if self.dataset.has_record() {
                    let mut recs = Records::new();
                    recs.cpu = self.cpu.clone();
                    recs.os = self.os.clone();
                    let mut bench = Benchmark::new();
                    bench.bench = self.bench_name.clone();
                    bench.dataset.push(self.dataset.clone());
                    recs.benchmark = bench;
                    self.dataset.clear();
                    self.bench_name = line.to_string();
                    return Some(recs);
                } else {
                    self.bench_name = line.to_string();
                }
            }

            if self.lcounts >= 1 {
                self.lcounts += 1;
                let mut bench = BenchmarkResult::new();
                let bench_result = line.split_whitespace().collect::<Vec<&str>>();
                bench.exec_time = bench_result[0].to_string();
                bench.mem_usage = bench_result[1].to_string();
                bench.cpu_usage = bench_result[2].to_string();
                self.dataset.result.push(bench);
            }

            if !self.bench_name.is_empty() {
                if line.starts_with("Dataset") {
                    self.dataset.name = line.split(":").nth(1).unwrap().trim().to_string();
                    self.lcounts = 1;
                }

                if self.lcounts == 10 {
                    self.lcounts = 0;
                }
            }
        }

        if self.dataset.has_record() {
            let mut recs = Records::new();
            recs.cpu = self.cpu.clone();
            recs.os = self.os.clone();
            let mut bench = Benchmark::new();
            bench.bench = self.bench_name.clone();
            bench.dataset.push(self.dataset.clone());
            recs.benchmark = bench;
            self.dataset.clear();
            self.bench_name = String::new();
            Some(recs)
        } else {
            None
        }
    }
}

impl<R: Read> Iterator for BenchReader<R> {
    type Item = Records;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_record()
    }
}

#[derive(Debug)]
struct Benchmark {
    bench: String,
    dataset: Vec<Dataset>,
}

impl Benchmark {
    fn new() -> Self {
        Self {
            bench: String::new(),
            dataset: Vec::new(),
        }
    }
}

#[derive(Debug)]
struct Records {
    cpu: String,
    os: String,
    benchmark: Benchmark,
}

impl Records {
    fn new() -> Self {
        Self {
            cpu: String::new(),
            os: String::new(),
            benchmark: Benchmark::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct Dataset {
    name: String,
    result: Vec<BenchmarkResult>,
}

impl Dataset {
    fn new() -> Self {
        Self {
            name: String::new(),
            result: Vec::new(),
        }
    }

    fn has_record(&self) -> bool {
        !self.name.is_empty() && !self.result.is_empty()
    }

    fn clear(&mut self) {
        self.name.clear();
        self.result.clear();
    }
}

#[derive(Debug, Clone)]
struct BenchmarkResult {
    exec_time: String,
    mem_usage: String,
    cpu_usage: String,
}

impl BenchmarkResult {
    fn new() -> Self {
        Self {
            exec_time: String::new(),
            mem_usage: String::new(),
            cpu_usage: String::new(),
        }
    }
}
