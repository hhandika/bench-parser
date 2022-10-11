use std::fs;
use std::io::Result;
use std::io::{prelude::*, BufWriter};
use std::path::PathBuf;
use std::{fs::File, io::BufReader, path::Path};

use chrono::NaiveTime;
use lazy_static::lazy_static;
use regex::Regex;

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
            self.parse_file(f, &mut writer)
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
            "Apps,Version,\
            Pubs,Datasets,ntax,alignment_counts,site_counts,\
            Datatype,Analyses,Platform,OS_name,CPU,Benchmark_dates,Latest_bench,\
            Execution_time,RAM_usage_kb,percent_CPU_usage,\
            execution_time_secs,RAM_usage_Mb\
        "
        )?;
        Ok(writer)
    }

    fn parse_file<W: Write>(&self, input: &Path, writer: &mut W) -> Result<()> {
        let file = File::open(input)?;
        let reader = BufReader::new(file);
        let records = BenchReader::new(reader);
        let file_stem = input
            .file_stem()
            .expect("Failed parsing file stem")
            .to_str()
            .expect("Failed parsing file stem to str");
        let analysis = self.parse_analysis_name(file_stem);
        let date = parse_date(file_stem);
        for rec in records {
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
                        write!(writer, "{} ({}),", pubs.pubs.name, pubs.pubs.datatype)?;
                        write!(writer, "{},", pubs.pubs.ntax)?;
                        write!(writer, "{},", pubs.pubs.aln_counts)?;
                        write!(writer, "{},", pubs.pubs.site_counts)?;
                        write!(writer, "{},", pubs.pubs.datatype)?;
                        write!(writer, "{},", self.match_analyses(analysis))?;
                        write!(writer, "{},", parse_platform(&rec.cpu))?;
                        write!(writer, "{},", rec.os)?;
                        write!(writer, "{},", rec.cpu)?;
                        write!(writer, "{},", date)?;
                        write!(writer, "TRUE,")?;
                        write!(writer, "{},", bench.exec_time)?;
                        write!(writer, "{},", bench.mem_usage)?;
                        write!(writer, "{},", bench.cpu_usage.replace('%', ""))?;
                        write!(writer, "{},", self.parse_time_to_secs(&bench.exec_time))?;
                        write!(writer, "{}", self.convert_kb_to_mb(&bench.mem_usage))?;
                        writeln!(writer)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn convert_kb_to_mb(&self, kb: &str) -> String {
        let mb = kb.parse::<f64>().expect("Failed parsing kb to f64") / 1024.0;
        mb.to_string()
    }

    fn parse_time_to_secs(&self, exe_time: &str) -> String {
        let count = exe_time.matches(':').count();
        let formatted_time = if count == 1 {
            format!("00:{}", exe_time)
        } else {
            exe_time.to_string()
        };
        let time =
            NaiveTime::parse_from_str(&formatted_time, "%H:%M:%S%.f").expect("Failed parsing time");
        time.format("%S%.f").to_string()
    }

    fn parse_analysis_name(&self, input: &'a str) -> &'a str {
        input
            .split('_')
            .next()
            .expect("Failed parsing analysis name")
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
                apps.version = format!("v{}", version);
            }
            app if app.contains("AMAS") => {
                if app.contains("aligned") {
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
            app if app.contains("goalign") => {
                apps.name = String::from("goalign");
                apps.version = String::from("v0.3.5");
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
        self.pubs.datatype = String::from("AA");
    }

    fn pub_shen(&mut self) {
        self.pubs.name = String::from("Shen et al. 2018");
        self.pubs.ntax = 343;
        self.pubs.aln_counts = 2408;
        self.pubs.site_counts = 1162805;
        self.pubs.datatype = String::from("AA");
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
            self.match_line_keyword(&line);
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
                    self.dataset.name = self.capture_name(&line);
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

    fn match_line_keyword(&mut self, line: &str) {
        match line {
            line if line.starts_with("Model") => {
                self.cpu = self.capture_name(line);
                self.os = String::from("Linux");
            }
            line if line.starts_with("Darwin") => {
                self.cpu = String::from("Apple M1");
                self.os = String::from("macOS");
            }
            line if line.contains("Microsoft") => self.os = String::from("Windows (WSL)"),
            line if line.contains("X86_64") => {
                self.os = String::from("macOS (Mb Air)");
                self.cpu = String::from("Intel Core i5-4260U")
            }
            line if line.starts_with("Benchmarking") => {
                self.bench_name = line.to_string();
            }
            line if line.starts_with("segul") => {
                self.segul_version = line.split_whitespace().nth(1).unwrap().to_string();
            }
            _ => (),
        }
    }

    fn capture_name(&self, line: &str) -> String {
        line.split(':')
            .nth(1)
            .expect("File capturing name")
            .trim()
            .to_string()
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

fn parse_date(file_stem: &str) -> String {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"(?P<y>\d{4})-(?P<m>\d{2})-(?P<d>\d{2})").expect("Failed to compile regex");
    };

    match RE.captures(file_stem) {
        Some(caps) => RE.replace_all(&caps[0], "$m/$d/$y").to_string(),
        None => String::from(file_stem),
    }
}

fn parse_platform(cpu_model: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"i(\d{1})-(\d{4}U)").expect("Failed to compile regex");
    };

    if RE.is_match(cpu_model) {
        String::from("Laptop")
    } else {
        String::from("Desktop")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date() {
        let file_stem = "concat_bench_raw_aa_OpenSUSE_2022-10-04.txt";
        let date = parse_date(file_stem);
        assert_eq!(date, "10/04/2022");
    }

    #[test]
    fn test_parse_platform() {
        let cpu_model = "Intel(R) Core(TM) i5-4260U CPU @ 1.40GHz";
        let platform = parse_platform(cpu_model);
        assert_eq!(platform, "Laptop");
    }

    #[test]
    fn parse_time_to_secs() {
        let path = [PathBuf::from(".")];
        let parser = Parser::new(&path, Path::new("results.csv"));
        let time = parser.parse_time_to_secs("00:42.0");
        assert_eq!(time, "42");
    }
}
