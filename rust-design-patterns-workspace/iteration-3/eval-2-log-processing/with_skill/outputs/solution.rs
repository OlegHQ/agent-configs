// Log Processing Framework
//
// Architecture: Three independent change vectors, each behind a trait.
//   - Sources:      Strategy pattern -- each source reads log lines from a different origin
//   - Processors:   Chain of Responsibility / Pipeline -- Vec<Box<dyn Processor>>
//   - Destinations: Strategy pattern -- each destination ships records to a different sink
//
// Adding a new source, processor, or destination means adding one new struct + impl.
// No existing code is modified.

use std::collections::HashMap;
use std::fmt;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::SystemTime;

// ---------------------------------------------------------------------------
// Core data types
// ---------------------------------------------------------------------------

/// Severity levels for log records. This is a small, closed set -- enum is correct here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Severity::Trace => "TRACE",
            Severity::Debug => "DEBUG",
            Severity::Info => "INFO",
            Severity::Warn => "WARN",
            Severity::Error => "ERROR",
            Severity::Fatal => "FATAL",
        };
        write!(f, "{}", s)
    }
}

impl Severity {
    pub fn parse(s: &str) -> Option<Severity> {
        match s.to_uppercase().as_str() {
            "TRACE" => Some(Severity::Trace),
            "DEBUG" => Some(Severity::Debug),
            "INFO" => Some(Severity::Info),
            "WARN" | "WARNING" => Some(Severity::Warn),
            "ERROR" | "ERR" => Some(Severity::Error),
            "FATAL" => Some(Severity::Fatal),
            _ => None,
        }
    }
}

/// A structured log record produced by parsing a raw log line.
#[derive(Debug, Clone)]
pub struct LogRecord {
    pub timestamp: Option<SystemTime>,
    pub severity: Severity,
    pub message: String,
    pub fields: HashMap<String, String>,
    pub raw: String,
}

impl LogRecord {
    pub fn new(severity: Severity, message: String) -> Self {
        Self {
            timestamp: Some(SystemTime::now()),
            severity,
            message,
            fields: HashMap::new(),
            raw: String::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Source trait -- Strategy pattern
// ---------------------------------------------------------------------------

/// A log source produces an iterator of raw log lines.
/// Each source owns its configuration and connection state.
///
/// Pattern: **Strategy** -- each source is an interchangeable algorithm for
/// reading log lines from a different origin.
pub trait Source {
    /// Human-readable name for diagnostics.
    fn name(&self) -> &str;

    /// Read all available lines. Returns a batch of raw strings.
    fn read_lines(&mut self) -> io::Result<Vec<String>>;
}

// --- File source ---

pub struct FileSource {
    path: PathBuf,
}

impl FileSource {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl Source for FileSource {
    fn name(&self) -> &str {
        "file"
    }

    fn read_lines(&mut self) -> io::Result<Vec<String>> {
        let file = std::fs::File::open(&self.path)?;
        let reader = BufReader::new(file);
        reader.lines().collect()
    }
}

// --- Stdin source ---

pub struct StdinSource;

impl Source for StdinSource {
    fn name(&self) -> &str {
        "stdin"
    }

    fn read_lines(&mut self) -> io::Result<Vec<String>> {
        let stdin = io::stdin();
        let reader = stdin.lock();
        reader.lines().collect()
    }
}

// --- Network socket source ---

pub struct TcpSource {
    addr: String,
}

impl TcpSource {
    pub fn new(addr: impl Into<String>) -> Self {
        Self { addr: addr.into() }
    }
}

impl Source for TcpSource {
    fn name(&self) -> &str {
        "tcp"
    }

    fn read_lines(&mut self) -> io::Result<Vec<String>> {
        let stream = TcpStream::connect(&self.addr)?;
        let reader = BufReader::new(stream);
        reader.lines().collect()
    }
}

// ---------------------------------------------------------------------------
// Processor trait -- Chain of Responsibility / Pipeline pattern
// ---------------------------------------------------------------------------

/// A single step in the log processing pipeline.
/// Each processor transforms a batch of records, potentially filtering,
/// enriching, or mutating them.
///
/// Pattern: **Chain of Responsibility** -- processors are composed into a
/// `Vec<Box<dyn Processor>>` and executed sequentially. Each step is
/// independent, testable in isolation, and new steps are added without
/// touching existing ones.
pub trait Processor {
    /// Human-readable name for diagnostics.
    fn name(&self) -> &str;

    /// Process a batch of records. May filter, mutate, or produce new records.
    fn process(&mut self, records: Vec<LogRecord>) -> Vec<LogRecord>;
}

// --- Filter by severity ---

pub struct SeverityFilter {
    min_severity: Severity,
}

impl SeverityFilter {
    pub fn new(min_severity: Severity) -> Self {
        Self { min_severity }
    }
}

impl Processor for SeverityFilter {
    fn name(&self) -> &str {
        "severity_filter"
    }

    fn process(&mut self, records: Vec<LogRecord>) -> Vec<LogRecord> {
        records
            .into_iter()
            .filter(|r| r.severity >= self.min_severity)
            .collect()
    }
}

// --- Redact sensitive fields ---

pub struct FieldRedactor {
    sensitive_fields: Vec<String>,
    replacement: String,
}

impl FieldRedactor {
    pub fn new(fields: Vec<String>) -> Self {
        Self {
            sensitive_fields: fields,
            replacement: "[REDACTED]".to_string(),
        }
    }

    pub fn with_replacement(mut self, replacement: impl Into<String>) -> Self {
        self.replacement = replacement.into();
        self
    }
}

impl Processor for FieldRedactor {
    fn name(&self) -> &str {
        "field_redactor"
    }

    fn process(&mut self, records: Vec<LogRecord>) -> Vec<LogRecord> {
        records
            .into_iter()
            .map(|mut r| {
                for field in &self.sensitive_fields {
                    if r.fields.contains_key(field) {
                        r.fields.insert(field.clone(), self.replacement.clone());
                    }
                }
                r
            })
            .collect()
    }
}

// --- Geo-IP enrichment ---

pub struct GeoIpEnricher {
    /// In a real system, this would hold a MaxMind database handle.
    db_path: PathBuf,
}

impl GeoIpEnricher {
    pub fn new(db_path: impl Into<PathBuf>) -> Self {
        Self {
            db_path: db_path.into(),
        }
    }

    fn lookup(&self, ip: &str) -> Option<String> {
        // Placeholder: a real implementation would query the GeoIP database.
        let _ = (&self.db_path, ip);
        Some("US".to_string())
    }
}

impl Processor for GeoIpEnricher {
    fn name(&self) -> &str {
        "geo_ip_enricher"
    }

    fn process(&mut self, records: Vec<LogRecord>) -> Vec<LogRecord> {
        records
            .into_iter()
            .map(|mut r| {
                if let Some(ip) = r.fields.get("ip").cloned() {
                    if let Some(country) = self.lookup(&ip) {
                        r.fields.insert("geo_country".to_string(), country);
                    }
                }
                r
            })
            .collect()
    }
}

// --- Metrics aggregator ---

pub struct MetricsAggregator {
    counts: HashMap<Severity, u64>,
    total: u64,
}

impl MetricsAggregator {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
            total: 0,
        }
    }

    /// Retrieve current metrics snapshot.
    pub fn snapshot(&self) -> (&HashMap<Severity, u64>, u64) {
        (&self.counts, self.total)
    }
}

impl Processor for MetricsAggregator {
    fn name(&self) -> &str {
        "metrics_aggregator"
    }

    fn process(&mut self, records: Vec<LogRecord>) -> Vec<LogRecord> {
        for r in &records {
            *self.counts.entry(r.severity).or_insert(0) += 1;
            self.total += 1;
        }
        // Pass-through: aggregation is a side effect, records are not consumed.
        records
    }
}

// --- Sampler ---

pub struct Sampler {
    /// Keep 1 out of every `rate` records.
    rate: u64,
    counter: u64,
}

impl Sampler {
    pub fn new(rate: u64) -> Self {
        assert!(rate > 0, "sample rate must be >= 1");
        Self { rate, counter: 0 }
    }
}

impl Processor for Sampler {
    fn name(&self) -> &str {
        "sampler"
    }

    fn process(&mut self, records: Vec<LogRecord>) -> Vec<LogRecord> {
        records
            .into_iter()
            .filter(|_| {
                self.counter += 1;
                self.counter % self.rate == 0
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Destination trait -- Strategy pattern
// ---------------------------------------------------------------------------

/// A log destination receives processed records and ships them somewhere.
///
/// Pattern: **Strategy** -- each destination is an interchangeable algorithm
/// for writing records to a different sink.
pub trait Destination {
    /// Human-readable name for diagnostics.
    fn name(&self) -> &str;

    /// Ship a batch of records. Implementations handle serialization and I/O.
    fn ship(&mut self, records: &[LogRecord]) -> io::Result<()>;

    /// Flush any buffered data. Called at the end of a pipeline run.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

// --- Stdout destination ---

pub struct StdoutDestination;

impl Destination for StdoutDestination {
    fn name(&self) -> &str {
        "stdout"
    }

    fn ship(&mut self, records: &[LogRecord]) -> io::Result<()> {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        for r in records {
            writeln!(out, "[{}] {}", r.severity, r.message)?;
        }
        Ok(())
    }
}

// --- File destination ---

pub struct FileDestination {
    path: PathBuf,
}

impl FileDestination {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl Destination for FileDestination {
    fn name(&self) -> &str {
        "file"
    }

    fn ship(&mut self, records: &[LogRecord]) -> io::Result<()> {
        use std::fs::OpenOptions;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        for r in records {
            writeln!(file, "[{}] {}", r.severity, r.message)?;
        }
        Ok(())
    }
}

// --- Elasticsearch destination ---

pub struct ElasticsearchDestination {
    url: String,
    index: String,
    buffer: Vec<String>,
}

impl ElasticsearchDestination {
    pub fn new(url: impl Into<String>, index: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            index: index.into(),
            buffer: Vec::new(),
        }
    }
}

impl Destination for ElasticsearchDestination {
    fn name(&self) -> &str {
        "elasticsearch"
    }

    fn ship(&mut self, records: &[LogRecord]) -> io::Result<()> {
        // Placeholder: a real implementation would HTTP POST to the bulk API.
        for r in records {
            let doc = format!(
                r#"{{"index":"{}","severity":"{}","message":"{}"}}"#,
                self.index, r.severity, r.message
            );
            self.buffer.push(doc);
        }
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            // Placeholder: send self.buffer as a bulk request to self.url.
            let _ = &self.url;
            self.buffer.clear();
        }
        Ok(())
    }
}

// --- Kafka destination ---

pub struct KafkaDestination {
    brokers: String,
    topic: String,
}

impl KafkaDestination {
    pub fn new(brokers: impl Into<String>, topic: impl Into<String>) -> Self {
        Self {
            brokers: brokers.into(),
            topic: topic.into(),
        }
    }
}

impl Destination for KafkaDestination {
    fn name(&self) -> &str {
        "kafka"
    }

    fn ship(&mut self, records: &[LogRecord]) -> io::Result<()> {
        // Placeholder: a real implementation would use a Kafka producer client.
        for r in records {
            let _payload = format!("[{}] {}", r.severity, r.message);
            let _ = (&self.brokers, &self.topic);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Parser -- converts raw lines into structured LogRecord values
// ---------------------------------------------------------------------------

/// Parses raw log lines into structured records.
/// Uses a simple default format; replace with a trait if multiple formats are needed.
pub fn parse_line(raw: &str) -> LogRecord {
    // Simple parser: expects "[SEVERITY] message" or falls back to Info.
    let trimmed = raw.trim();
    if let Some(rest) = trimmed.strip_prefix('[') {
        if let Some(bracket_end) = rest.find(']') {
            let level_str = &rest[..bracket_end];
            let message = rest[bracket_end + 1..].trim().to_string();
            if let Some(severity) = Severity::parse(level_str) {
                return LogRecord {
                    timestamp: Some(SystemTime::now()),
                    severity,
                    message,
                    fields: HashMap::new(),
                    raw: raw.to_string(),
                };
            }
        }
    }
    // Fallback: treat entire line as an Info message.
    LogRecord {
        timestamp: Some(SystemTime::now()),
        severity: Severity::Info,
        message: trimmed.to_string(),
        fields: HashMap::new(),
        raw: raw.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Pipeline -- Builder pattern for composing source + processors + destinations
// ---------------------------------------------------------------------------

/// The processing pipeline: reads from a source, runs records through a chain
/// of processors, and ships results to one or more destinations.
///
/// Pattern: **Builder** -- the pipeline is constructed incrementally by adding
/// sources, processors, and destinations. `run()` executes the composed pipeline.
pub struct Pipeline {
    sources: Vec<Box<dyn Source>>,
    processors: Vec<Box<dyn Processor>>,
    destinations: Vec<Box<dyn Destination>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            processors: Vec::new(),
            destinations: Vec::new(),
        }
    }

    /// Add a log source. Multiple sources can be added; all are read during a run.
    pub fn source(mut self, source: impl Source + 'static) -> Self {
        self.sources.push(Box::new(source));
        self
    }

    /// Append a processor to the pipeline. Processors run in the order they are added.
    pub fn processor(mut self, processor: impl Processor + 'static) -> Self {
        self.processors.push(Box::new(processor));
        self
    }

    /// Add a destination. Records are shipped to all destinations after processing.
    pub fn destination(mut self, dest: impl Destination + 'static) -> Self {
        self.destinations.push(Box::new(dest));
        self
    }

    /// Execute the pipeline: read -> parse -> process -> ship.
    pub fn run(&mut self) -> io::Result<PipelineResult> {
        // 1. Read raw lines from all sources.
        let mut raw_lines = Vec::new();
        for source in &mut self.sources {
            match source.read_lines() {
                Ok(lines) => raw_lines.extend(lines),
                Err(e) => {
                    eprintln!("Source '{}' failed: {}", source.name(), e);
                }
            }
        }

        // 2. Parse raw lines into structured records.
        let mut records: Vec<LogRecord> =
            raw_lines.iter().map(|line| parse_line(line)).collect();

        let parsed_count = records.len();

        // 3. Run records through the processor chain.
        for processor in &mut self.processors {
            records = processor.process(records);
        }

        let output_count = records.len();

        // 4. Ship to all destinations.
        for dest in &mut self.destinations {
            if let Err(e) = dest.ship(&records) {
                eprintln!("Destination '{}' failed: {}", dest.name(), e);
            }
        }

        // 5. Flush all destinations.
        for dest in &mut self.destinations {
            if let Err(e) = dest.flush() {
                eprintln!("Destination '{}' flush failed: {}", dest.name(), e);
            }
        }

        Ok(PipelineResult {
            lines_read: raw_lines.len(),
            records_parsed: parsed_count,
            records_output: output_count,
        })
    }
}

/// Summary of a pipeline execution.
#[derive(Debug)]
pub struct PipelineResult {
    pub lines_read: usize,
    pub records_parsed: usize,
    pub records_output: usize,
}

impl fmt::Display for PipelineResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Pipeline: {} lines read, {} parsed, {} output",
            self.lines_read, self.records_parsed, self.records_output
        )
    }
}

// ---------------------------------------------------------------------------
// Example usage and tests
// ---------------------------------------------------------------------------

/// Demonstrates building and running a pipeline.
///
/// ```no_run
/// let result = Pipeline::new()
///     .source(FileSource::new("/var/log/app.log"))
///     .processor(SeverityFilter::new(Severity::Warn))
///     .processor(FieldRedactor::new(vec!["password".into(), "ssn".into()]))
///     .processor(Sampler::new(10))
///     .destination(StdoutDestination)
///     .destination(ElasticsearchDestination::new("http://localhost:9200", "logs"))
///     .run()
///     .unwrap();
///
/// println!("{}", result);
/// ```
fn main() {
    // Build a pipeline using the builder API.
    let mut pipeline = Pipeline::new()
        .source(FileSource::new("/var/log/app.log"))
        .processor(SeverityFilter::new(Severity::Warn))
        .processor(FieldRedactor::new(vec![
            "password".to_string(),
            "ssn".to_string(),
        ]))
        .processor(MetricsAggregator::new())
        .processor(Sampler::new(5))
        .destination(StdoutDestination)
        .destination(FileDestination::new("/tmp/filtered.log"))
        .destination(ElasticsearchDestination::new(
            "http://localhost:9200",
            "app-logs",
        ))
        .destination(KafkaDestination::new("localhost:9092", "logs-topic"));

    match pipeline.run() {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Pipeline error: {}", e),
    }
}

// ---------------------------------------------------------------------------
// Unit tests -- each component is independently testable
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_records() -> Vec<LogRecord> {
        vec![
            LogRecord::new(Severity::Debug, "debug msg".into()),
            LogRecord::new(Severity::Info, "info msg".into()),
            LogRecord::new(Severity::Warn, "warn msg".into()),
            LogRecord::new(Severity::Error, "error msg".into()),
        ]
    }

    #[test]
    fn severity_filter_keeps_warn_and_above() {
        let mut filter = SeverityFilter::new(Severity::Warn);
        let result = filter.process(sample_records());
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].severity, Severity::Warn);
        assert_eq!(result[1].severity, Severity::Error);
    }

    #[test]
    fn field_redactor_replaces_sensitive_values() {
        let mut redactor = FieldRedactor::new(vec!["password".into()]);
        let mut record = LogRecord::new(Severity::Info, "login".into());
        record.fields.insert("password".into(), "secret123".into());
        record.fields.insert("user".into(), "alice".into());

        let result = redactor.process(vec![record]);
        assert_eq!(result[0].fields["password"], "[REDACTED]");
        assert_eq!(result[0].fields["user"], "alice");
    }

    #[test]
    fn sampler_keeps_every_nth_record() {
        let mut sampler = Sampler::new(2);
        let records = sample_records(); // 4 records
        let result = sampler.process(records);
        // Keeps records at counter 2 and 4 (every 2nd).
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn metrics_aggregator_counts_by_severity() {
        let mut aggregator = MetricsAggregator::new();
        let _result = aggregator.process(sample_records());
        let (counts, total) = aggregator.snapshot();
        assert_eq!(total, 4);
        assert_eq!(*counts.get(&Severity::Debug).unwrap(), 1);
        assert_eq!(*counts.get(&Severity::Error).unwrap(), 1);
    }

    #[test]
    fn geo_ip_enricher_adds_country_field() {
        let mut enricher = GeoIpEnricher::new("/path/to/GeoLite2.mmdb");
        let mut record = LogRecord::new(Severity::Info, "request".into());
        record.fields.insert("ip".into(), "8.8.8.8".into());

        let result = enricher.process(vec![record]);
        assert!(result[0].fields.contains_key("geo_country"));
    }

    #[test]
    fn parse_line_extracts_severity_and_message() {
        let record = parse_line("[ERROR] disk full");
        assert_eq!(record.severity, Severity::Error);
        assert_eq!(record.message, "disk full");
    }

    #[test]
    fn parse_line_defaults_to_info_on_unknown_format() {
        let record = parse_line("some random log line");
        assert_eq!(record.severity, Severity::Info);
        assert_eq!(record.message, "some random log line");
    }

    #[test]
    fn pipeline_chains_processors() {
        // Verify that processors compose: filter then sample.
        let mut filter = SeverityFilter::new(Severity::Warn);
        let mut sampler = Sampler::new(1); // keep all

        let records = sample_records();
        let after_filter = filter.process(records);
        let after_sample = sampler.process(after_filter);
        assert_eq!(after_sample.len(), 2); // only Warn + Error survive the filter
    }

    /// A test source that yields pre-canned lines.
    struct TestSource {
        lines: Vec<String>,
    }

    impl Source for TestSource {
        fn name(&self) -> &str {
            "test"
        }
        fn read_lines(&mut self) -> io::Result<Vec<String>> {
            Ok(self.lines.clone())
        }
    }

    /// A test destination that captures shipped records.
    struct CaptureDest {
        captured: Vec<String>,
    }

    impl CaptureDest {
        fn new() -> Self {
            Self {
                captured: Vec::new(),
            }
        }
    }

    impl Destination for CaptureDest {
        fn name(&self) -> &str {
            "capture"
        }
        fn ship(&mut self, records: &[LogRecord]) -> io::Result<()> {
            for r in records {
                self.captured.push(format!("[{}] {}", r.severity, r.message));
            }
            Ok(())
        }
    }

    #[test]
    fn full_pipeline_integration() {
        let source = TestSource {
            lines: vec![
                "[DEBUG] verbose stuff".into(),
                "[INFO] normal operation".into(),
                "[WARN] something off".into(),
                "[ERROR] bad thing happened".into(),
            ],
        };

        let mut pipeline = Pipeline::new()
            .source(source)
            .processor(SeverityFilter::new(Severity::Warn));

        // We need to capture output, so we add a destination after building.
        let dest = CaptureDest::new();
        pipeline.destinations.push(Box::new(dest));

        let result = pipeline.run().unwrap();
        assert_eq!(result.lines_read, 4);
        assert_eq!(result.records_parsed, 4);
        assert_eq!(result.records_output, 2); // only WARN + ERROR
    }
}
