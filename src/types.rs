#[derive(Debug)]
pub struct Benchmark {
    pub bench: String,
    pub dataset: Vec<Dataset>,
}

impl Benchmark {
    pub fn new() -> Self {
        Self {
            bench: String::new(),
            dataset: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Records {
    pub cpu: String,
    pub os: String,
    pub segul_version: String,
    pub benchmark: Benchmark,
}

impl Records {
    pub fn new() -> Self {
        Self {
            cpu: String::new(),
            os: String::new(),
            segul_version: String::new(),
            benchmark: Benchmark::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Dataset {
    pub name: String,
    pub result: Vec<BenchmarkResult>,
}

impl Dataset {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            result: Vec::new(),
        }
    }

    pub fn has_record(&self) -> bool {
        !self.name.is_empty() && !self.result.is_empty()
    }

    pub fn clear(&mut self) {
        self.name.clear();
        self.result.clear();
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub exec_time: String,
    pub mem_usage: String,
    pub cpu_usage: String,
}

impl BenchmarkResult {
    pub fn new() -> Self {
        Self {
            exec_time: String::new(),
            mem_usage: String::new(),
            cpu_usage: String::new(),
        }
    }
}

pub struct Apps {
    pub name: String,
    pub version: String,
}

impl Apps {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            version: String::new(),
        }
    }
}

pub struct Pubs {
    pub name: String,
    pub ntax: usize,
    pub aln_counts: usize,
    pub site_counts: usize,
    pub datatype: String,
}

impl Pubs {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            ntax: 0,
            aln_counts: 0,
            site_counts: 0,
            datatype: String::new(),
        }
    }
}
