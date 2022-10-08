use std::fs;
use std::io::Result;
use std::io::{prelude::*, BufWriter};
use std::path::PathBuf;
use std::{fs::File, io::BufReader, path::Path};

use crate::types::{Apps, Benchmark, BenchmarkResult, Dataset, Pubs, Records};

pub struct Parser<'a> {
    pub input: &'a [PathBuf],
    pub output: &'a Path,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [PathBuf], output: &'a Path) -> Self {
        Self { input, output }
    }

    pub fn parse_benchmark(&self) -> Result<()> {
        let mut writer = self.write_records().expect("Failed writing records");
        self.input.iter().for_each(|f| {
            let analysis = f
                .file_stem()
                .expect("Failed parsing file stem")
                .to_str()
                .expect("Failed parsing file stem to str")
                .split('_')
                .nth(0)
                .unwrap();
            self.parse_file(f, &mut writer, analysis)
                .expect("Failed to parse text");
        });
        Ok(())
    }

    fn write_records(&self) -> Result<BufWriter<File>> {
        let output = self.output.with_extension("csv");
        fs::create_dir_all(&output.parent().expect("Failed creating output directory"))?;
        let file = File::create(output)?;
        let mut writer = BufWriter::new(file);
        writeln!(
            writer,
            "Apps,Version,Pubs,ntax,alignment_counts,site_counts,Datatype,Analyses,OS,CPU,Execution_time,RAM_usage_kb,CPU_usage
        "
        )?;
        Ok(writer)
    }

    fn parse_file<W: Write>(&self, input: &Path, writer: &mut W, analysis: &str) -> Result<()> {
        let file = File::open(input)?;
        let reader = BufReader::new(file);
        let records = BenchReader::new(reader);
        for rec in records.into_iter() {
            for dataset in rec.benchmark.dataset {
                if dataset.result.len() != 10 {
                    panic!(
                        "Invalid dataset result length {}: {}",
                        rec.benchmark.bench, dataset.name
                    );
                } else {
                    for bench in dataset.result {
                        let apps = self.match_apps(&rec.benchmark.bench, &rec.segul_version);
                        let pubs = self.parse_pubs(&dataset.name);
                        write!(writer, "{},", apps.name)?;
                        write!(writer, "{},", apps.version)?;
                        write!(writer, "{},", pubs.pubs.name)?;
                        write!(writer, "{},", pubs.pubs.ntax)?;
                        write!(writer, "{},", pubs.pubs.aln_counts)?;
                        write!(writer, "{},", pubs.pubs.site_counts)?;
                        write!(writer, "{},", pubs.pubs.datatype)?;
                        write!(writer, "{},", self.match_analyses(analysis))?;
                        write!(writer, "{},", rec.os)?;
                        write!(writer, "{},", rec.cpu)?;
                        write!(writer, "{},", bench.exec_time)?;
                        write!(writer, "{},", bench.mem_usage)?;
                        write!(writer, "{}", bench.cpu_usage)?;
                        writeln!(writer)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn parse_pubs(&self, pubs: &str) -> PubRecord {
        let mut pub_recs = PubRecord::new();
        match pubs.to_lowercase() {
            pubs if pubs.contains("esselstyn") => pub_recs.pub_esselstyn(),
            pubs if pubs.contains("oliveros") => pub_recs.pub_oliveros(),
            pubs if pubs.contains("jarvis") => pub_recs.pub_jarvis(),
            pubs if pubs.contains("chan") => pub_recs.pub_chan(),
            pubs if pubs.contains("wu") => pub_recs.pub_wu(),
            pubs if pubs.contains("shen") => pub_recs.pub_shen(),
            _ => pub_recs.pub_unknown(pubs),
        }

        pub_recs
    }

    fn match_apps(&self, app: &str, version: &str) -> Apps {
        let mut apps = Apps::new();
        match app {
            app if app.contains("SEGUL") => {
                if app.contains("ignore") {
                    apps.name = String::from("SEGUL (--datatype ignore)");
                } else {
                    apps.name = String::from("SEGUL");
                }
                apps.version = String::from(version);
            }
            app if app.contains("AMAS") => {
                if app.contains("align") {
                    apps.name = String::from("AMAS (--check-align)");
                } else {
                    apps.name = String::from("AMAS");
                }
                apps.version = String::from("v1.02");
            }
            app if app.contains("Phyluce") => {
                apps.name = String::from("Phyluce");
                apps.version = String::from("v1.7.1");
            }
            _ => {
                apps.name = String::from(app);
                apps.version = String::from("Unknown");
            }
        };
        apps
    }

    fn match_analyses(&self, analysis: &str) -> String {
        match analysis {
            "concat" => "Alignment Concatenation".to_string(),
            "convert" => "Alignment Conversion".to_string(),
            "summary" => "Summary Statistics".to_string(),
            "remove" => "Sequence Removal".to_string(),
            "split" => "Alignment Splitting".to_string(),
            _ => analysis.to_string(),
        }
    }
}

struct PubRecord {
    pubs: Pubs,
}

impl PubRecord {
    fn new() -> Self {
        Self { pubs: Pubs::new() }
    }

    fn pub_esselstyn(&mut self) {
        self.pubs.name = String::from("Esselstyn et al. 2021");
        self.pubs.ntax = 102;
        self.pubs.aln_counts = 4040;
        self.pubs.site_counts = 5398947;
        self.pubs.datatype = String::from("DNA");
    }

    fn pub_oliveros(&mut self) {
        self.pubs.name = String::from("Oliveros et al. 2019");
        self.pubs.ntax = 221;
        self.pubs.aln_counts = 4060;
        self.pubs.site_counts = 2464926;
        self.pubs.datatype = String::from("DNA");
    }

    fn pub_jarvis(&mut self) {
        self.pubs.name = String::from("Jarvis et al. 2014");
        self.pubs.ntax = 49;
        self.pubs.aln_counts = 3679;
        self.pubs.site_counts = 9251694;
        self.pubs.datatype = String::from("DNA");
    }

    fn pub_chan(&mut self) {
        self.pubs.name = String::from("Chan et al. 2020");
        self.pubs.ntax = 50;
        self.pubs.aln_counts = 13181;
        self.pubs.site_counts = 6180393;
        self.pubs.datatype = String::from("DNA");
    }

    fn pub_wu(&mut self) {
        self.pubs.name = String::from("Wu et al. 2018");
        self.pubs.ntax = 90;
        self.pubs.aln_counts = 5162;
        self.pubs.site_counts = 3050198;
        self.pubs.datatype = String::from("DNA");
    }

    fn pub_shen(&mut self) {
        self.pubs.name = String::from("Shen et al. 2018");
        self.pubs.ntax = 343;
        self.pubs.aln_counts = 2408;
        self.pubs.site_counts = 1162805;
        self.pubs.datatype = String::from("DNA");
    }

    fn pub_unknown(&mut self, pubs: &str) {
        self.pubs.name = pubs.to_string();
        self.pubs.ntax = 0;
        self.pubs.aln_counts = 0;
        self.pubs.site_counts = 0;
        self.pubs.datatype = String::from("UNKNOWN");
    }
}

struct BenchReader<R: Read> {
    reader: BufReader<R>,
    cpu: String,
    os: String,
    bench_name: String,
    segul_version: String,
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
            segul_version: String::new(),
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
            } else if line.starts_with("segul") {
                self.segul_version = line.trim().split_whitespace().nth(1).unwrap().to_string();
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
                    let recs = self.parse_records();
                    self.lcounts = 0;
                    return Some(recs);
                }
            }
        }

        if self.dataset.has_record() {
            let recs = self.parse_records();
            self.bench_name.clear();
            Some(recs)
        } else {
            None
        }
    }

    fn parse_records(&mut self) -> Records {
        let mut recs = Records::new();
        recs.cpu = self.cpu.clone();
        recs.os = self.os.clone();
        recs.segul_version = self.segul_version.clone();
        let mut bench = Benchmark::new();
        bench.bench = self.bench_name.clone();
        bench.dataset.push(self.dataset.clone());
        recs.benchmark = bench;
        self.dataset.clear();

        recs
    }
}

impl<R: Read> Iterator for BenchReader<R> {
    type Item = Records;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_record()
    }
}
