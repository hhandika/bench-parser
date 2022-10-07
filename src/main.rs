use std::io::Result;
use std::io::{prelude::*, BufWriter};
use std::path::PathBuf;
use std::{fs::File, io::BufReader, path::Path};

fn main() {
    let path = "data/*.txt";
    let files = glob::glob(path)
        .expect("Failed globbing files")
        .filter_map(|ok| ok.ok())
        .collect::<Vec<PathBuf>>();

    let mut writer = write_records().expect("Failed writing records");
    files.iter().for_each(|f| {
        let analysis = f
            .file_stem()
            .expect("Failed parsing file stem")
            .to_str()
            .expect("Failed parsing file stem to str")
            .split('_')
            .nth(0)
            .unwrap();
        parse_text(f, &mut writer, analysis).expect("Failed to parse text");
    })
}

fn write_records() -> Result<BufWriter<File>> {
    let file = File::create("data/result.csv")?;
    let mut writer = BufWriter::new(file);
    writeln!(
        writer,
        "Benchmark,type,OS,CPU,Dataset,Execution_time,RAM_usage_kb,CPU_usage
    "
    )?;
    Ok(writer)
}

fn parse_text<W: Write>(input: &Path, writer: &mut W, analysis: &str) -> Result<()> {
    let file = File::open(input).unwrap();
    let reader = BufReader::new(file);
    let records = BenchReader::new(reader);
    for rec in records.into_iter() {
        for res in rec.benchmark.dataset {
            for bench in res.result {
                write!(writer, "{},", analysis)?;
                write!(writer, "{},", rec.benchmark.bench)?;
                write!(writer, "{},", rec.os)?;
                write!(writer, "{},", rec.cpu)?;
                write!(writer, "{},", res.name)?;
                write!(writer, "{},", bench.exec_time)?;
                write!(writer, "{},", bench.mem_usage)?;
                write!(writer, "{}", bench.cpu_usage)?;
                writeln!(writer)?;
            }
        }
    }

    Ok(())
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
                self.bench_name = line.to_string();
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
                    self.lcounts = 1;
                    self.dataset.name = line.split(":").nth(1).unwrap().trim().to_string();
                }

                if self.lcounts > 10 && !line.trim().is_empty() {
                    let mut recs = Records::new();
                    recs.cpu = self.cpu.clone();
                    recs.os = self.os.clone();
                    let mut bench = Benchmark::new();
                    bench.bench = self.bench_name.clone();
                    bench.dataset.push(self.dataset.clone());
                    recs.benchmark = bench;
                    self.dataset.clear();
                    self.lcounts = 0;
                    return Some(recs);
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
