use std::fs;
use std::io::Result;
use std::io::{prelude::*, BufWriter};
use std::path::PathBuf;
use std::{fs::File, io::BufReader, path::Path};

use lazy_static::lazy_static;
use regex::Regex;

use crate::types::{Apps, Benchmark, BenchmarkResult, Dataset, Pubs, Records};

pub struct Parser<'a> {
    pub input: &'a [PathBuf],
    pub output: &'a Path,
    pub dataset_size: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [PathBuf], output: &'a Path, dataset_size: usize) -> Self {
        Self {
            input,
            output,
            dataset_size,
        }
    }

    pub fn parse_benchmark(&self) -> Result<()> {
        let mut writer = self.write_records().expect("Failed writing records");
        self.print_input();
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
            Pubs,Datasets,NTAX,Character_counts,Alignment_counts,Site_counts,\
            Datatype,Analyses,Platform,OS_name,CPU,Benchmark_dates,Latest_bench,\
            Execution_time,RAM_usage_kb,Percent_CPU_usage,\
            Execution_time_secs,RAM_usage_Mb\
        "
        )?;
        Ok(writer)
    }

    fn parse_file<W: Write>(&self, input: &Path, writer: &mut W) -> Result<()> {
        let file = File::open(input)?;
        let reader = BufReader::new(file);
        let records = BenchReader::new(reader, self.dataset_size);
        let file_stem = input
            .file_stem()
            .expect("Failed parsing file stem")
            .to_str()
            .expect("Failed parsing file stem to str");
        let analysis = self.parse_analysis_name(file_stem);
        let analysis_name = self.match_analyses(analysis);
        let date = parse_date(&file_stem);
        for rec in records {
            for dataset in rec.benchmark.dataset {
                let dataset_size = dataset.result.len();
                if dataset_size != self.dataset_size || !dataset.has_record() {
                    panic!(
                        "Invalid dataset result of {} for {}. \
                        Expected {} records. Found : {}. Offending file: {}",
                        rec.benchmark.bench,
                        dataset.name,
                        self.dataset_size,
                        dataset_size,
                        input.display()
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
                        write!(writer, "{},", pubs.pubs.char_counts)?;
                        write!(writer, "{},", pubs.pubs.aln_counts)?;
                        write!(writer, "{},", pubs.pubs.site_counts)?;
                        write!(writer, "{},", pubs.pubs.datatype)?;
                        write!(writer, "{},", analysis_name)?;
                        write!(
                            writer,
                            "{},",
                            self.parse_platform_with_app_name(&rec.cpu, &apps.name)
                        )?;
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
        println!("Finished parsing {} as {}", input.display(), analysis_name);

        Ok(())
    }

    fn parse_platform_with_app_name(&self, cpu_model: &str, app: &str) -> String {
        if app.contains("GUI") {
            // capture the word inside the parenthesis
            let name = app.split_whitespace().collect::<Vec<&str>>();
            assert!(name.len() == 3, "Invalid app name {}", app);
            return format!("GUI ({})", name[2]);
        }

        parse_platform(cpu_model)
    }

    fn convert_kb_to_mb(&self, kb: &str) -> f32 {
        kb.parse::<f32>().expect("Failed parsing kb to f64") / 1024.0
    }

    fn parse_time_to_secs(&self, exe_time: &str) -> f64 {
        let splitted_time: Vec<&str> = exe_time.split(':').collect();
        match splitted_time.len() {
            1 => splitted_time[0].parse().expect("Failed parsing seconds"),
            2 => {
                splitted_time[0]
                    .parse::<f64>()
                    .expect("Failed parsing minutes")
                    * 60.0
                    + splitted_time[1]
                        .parse::<f64>()
                        .expect("Failed parsing seconds")
            }
            3 => {
                splitted_time[0]
                    .parse::<f64>()
                    .expect("Failed parsing hours")
                    * 3600.0
                    + splitted_time[1]
                        .parse::<f64>()
                        .expect("Failed parsing minutes")
                        * 60.0
                    + splitted_time[2]
                        .parse::<f64>()
                        .expect("Failed parsing seconds")
            }
            _ => panic!("Invalid time format"),
        }
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
            pubs if pubs.contains("SRR26062012") => pub_recs.pub_genome(),
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
                } else if app.contains("GUI") {
                    let name = app.split_whitespace().collect::<Vec<&str>>();
                    assert!(name.len() == 4, "Invalid app name {}", app);
                    apps.name = format!("SEGUL GUI ({})", name[3]);
                } else {
                    apps.name = String::from("SEGUL");
                }
                apps.version = format!("v{}", version);
            }
            app if app.contains("AMAS") => {
                if app.contains("align") {
                    apps.name = String::from("AMAS (--check-align)");
                } else if app.contains("--remove-empty") {
                    apps.name = String::from("AMAS (--remove-empty)");
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
                if app.contains("multi-core") {
                    apps.name = String::from("goalign (multi-core)");
                } else {
                    apps.name = String::from("goalign (single-core)");
                }
            }
            _ => {
                apps.name = String::from(app);
                apps.version = String::from("Unknown");
            }
        };
        apps
    }

    fn match_analyses(&self, analysis: &str) -> String {
        let dataset_format = self.parse_dataset_format(analysis);
        let analysis = match dataset_format.0.as_str() {
            "concat" => "Alignment Concatenation".to_string(),
            "convert" => "Alignment Conversion".to_string(),
            "summary" => "Alignment Summary".to_string(),
            "remove" => "Sequence Removal".to_string(),
            "split" => "Alignment Splitting".to_string(),
            "raw" => "Read Summary".to_string(),
            _ => analysis.to_string(),
        };
        let dataset = dataset_format.1;
        match dataset {
            Some(dataset) => format!("{} ({})", analysis, dataset.to_uppercase()),
            None => format!("{} (NEXUS)", analysis),
        }
    }

    fn parse_dataset_format(&self, dataset: &str) -> (String, Option<String>) {
        if dataset.contains('-') {
            let dataset = dataset.split('-').collect::<Vec<&str>>();
            (dataset[0].to_string(), Some(dataset[1].to_string()))
        } else {
            (dataset.to_string(), None)
        }
    }

    fn print_input(&self) {
        println!("File Counts: {}", self.input.len());
    }
}

struct PubRecord {
    pubs: Pubs,
}

impl PubRecord {
    fn new() -> Self {
        Self { pubs: Pubs::new() }
    }

    fn pub_chan(&mut self) {
        self.pubs.name = String::from("Chan et al. 2020");
        self.pubs.ntax = 50;
        self.pubs.aln_counts = 13181;
        self.pubs.char_counts = 239310808;
        self.pubs.site_counts = 6180393;
        self.pubs.datatype = String::from("DNA");
    }

    fn pub_esselstyn(&mut self) {
        self.pubs.name = String::from("Esselstyn et al. 2021");
        self.pubs.ntax = 102;
        self.pubs.aln_counts = 4040;
        self.pubs.char_counts = 358099656;
        self.pubs.site_counts = 5398947;
        self.pubs.datatype = String::from("DNA");
    }

    fn pub_jarvis(&mut self) {
        self.pubs.name = String::from("Jarvis et al. 2014");
        self.pubs.ntax = 49;
        self.pubs.aln_counts = 3679;
        self.pubs.char_counts = 453333006;
        self.pubs.site_counts = 9251694;
        self.pubs.datatype = String::from("DNA");
    }

    fn pub_oliveros(&mut self) {
        self.pubs.name = String::from("Oliveros et al. 2019");
        self.pubs.ntax = 221;
        self.pubs.aln_counts = 4060;
        self.pubs.char_counts = 522529858;
        self.pubs.site_counts = 2464926;
        self.pubs.datatype = String::from("DNA");
    }

    fn pub_shen(&mut self) {
        self.pubs.name = String::from("Shen et al. 2018");
        self.pubs.ntax = 343;
        self.pubs.aln_counts = 2408;
        self.pubs.char_counts = 398842115;
        self.pubs.site_counts = 1162805;
        self.pubs.datatype = String::from("AA");
    }

    fn pub_wu(&mut self) {
        self.pubs.name = String::from("Wu et al. 2018");
        self.pubs.ntax = 90;
        self.pubs.aln_counts = 5162;
        self.pubs.char_counts = 257060172;
        self.pubs.site_counts = 3050198;
        self.pubs.datatype = String::from("AA");
    }

    fn pub_genome(&mut self) {
        self.pubs.name = String::from("SRR26062012");
        self.pubs.ntax = 1;
        self.pubs.aln_counts = 0;
        self.pubs.char_counts = 243874896842;
        self.pubs.site_counts = 0;
        self.pubs.datatype = String::from("DNA");
    }

    fn pub_unknown(&mut self, pubs: &str) {
        self.pubs.name = pubs.to_string();
        self.pubs.ntax = 0;
        self.pubs.aln_counts = 0;
        self.pubs.site_counts = 0;
        self.pubs.datatype = String::from("Whole Genome");
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
    dataset_size: usize,
}

impl<R: Read> BenchReader<R> {
    fn new(reader: R, dataset_size: usize) -> Self {
        Self {
            reader: BufReader::new(reader),
            cpu: String::new(),
            os: String::new(),
            bench_name: String::new(),
            segul_version: String::new(),
            dataset: Dataset::new(),
            lcounts: 0,
            dataset_size: dataset_size,
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

                if self.lcounts > self.dataset_size && !line.trim().is_empty() {
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

    macro_rules! initialize_parser {
        ($parser: ident) => {
            let path = [PathBuf::from(".")];
            let $parser = Parser::new(&path, Path::new("results.csv"), 5);
        };
    }

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
        initialize_parser!(parser);
        let time = parser.parse_time_to_secs("00:42.0");
        let time_minute = parser.parse_time_to_secs("01:30.00");
        assert_eq!(time, 42.0);
        assert_eq!(time_minute, 90.0);
    }

    #[test]
    fn test_analysis_parsing() {
        initialize_parser!(parser);
        let file_name = "remove_bench_raw_aa_OpenSUSE_2022-10-04.txt";
        let name = parser.parse_analysis_name(file_name);
        assert_eq!("Sequence Removal (NEXUS)", parser.match_analyses(name));
    }

    #[test]
    fn test_bench_parsing() {
        let input = "tests/data/*.txt";
        let files: Vec<PathBuf> = glob::glob(input)
            .expect("Failed to read glob pattern")
            .filter_map(|ok| ok.ok())
            .collect();
        let parser = Parser::new(&files, Path::new("results.csv"), 5);
        let mut analysis = files
            .iter()
            .map(|f| parser.parse_analysis_name(f.file_name().unwrap().to_str().unwrap()))
            .map(|n| parser.match_analyses(n))
            .collect::<Vec<_>>();
        analysis.dedup();
        assert_eq!(5, analysis.len());
    }
}
