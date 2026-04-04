// =============================================================================
// Log Processing Framework
//
// Architecture:
//   Source (trait) --> Pipeline of Processor (trait) --> Destination (trait)
//
// Each dimension (source, processor, destination) is independently extensible
// via traits. Processors compose into chains. The Pipeline struct wires
// everything together.
// =============================================================================

use std::collections::HashMap;
use std::fmt;
use std::io::{self, BufRead, Write};
use std::net::TcpListener;
use std::time::{SystemTime, UNIX_EPOCH};

// =============================================================================
// Core data model
// =============================================================================

/// Severity levels for log records.
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
        match self {
            Severity::Trace => write!(f, "TRACE"),
            Severity::Debug => write!(f, "DEBUG"),
            Severity::Info => write!(f, "INFO"),
            Severity::Warn => write!(f, "WARN"),
            Severity::Error => write!(f, "ERROR"),
            Severity::Fatal => write!(f, "FATAL"),
        }
    }
}

impl Severity {
    pub fn parse(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "TRACE" => Severity::Trace,
            "DEBUG" => Severity::Debug,
            "INFO" => Severity::Info,
            "WARN" => Severity::Warn,
            "ERROR" => Severity::Error,
            "FATAL" => Severity::Fatal,
            _ => Severity::Info,
        }
    }
}

/// A structured log record flowing through the pipeline.
#[derive(Debug, Clone)]
pub struct LogRecord {
    pub timestamp: u64,
    pub severity: Severity,
    pub message: String,
    pub source: String,
    pub fields: HashMap<String, String>,
}

impl LogRecord {
    pub fn new(severity: Severity, message: impl Into<String>) -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Self {
            timestamp: ts,
            severity,
            message: message.into(),
            source: String::new(),
            fields: HashMap::new(),
        }
    }

    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }
}

impl fmt::Display for LogRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {} | {}", self.severity, self.timestamp, self.message)?;
        if !self.fields.is_empty() {
            let pairs: Vec<String> = self
                .fields
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            write!(f, " | {}", pairs.join(", "))?;
        }
        Ok(())
    }
}

// =============================================================================
// Simple line parser (turns raw text into LogRecord)
// =============================================================================

/// Parses a raw log line into a structured LogRecord.
///
/// Expected format: `SEVERITY message` or just `message` (defaults to INFO).
pub fn parse_log_line(line: &str) -> LogRecord {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return LogRecord::new(Severity::Info, "");
    }

    let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
    if parts.len() == 2 {
        let candidate = parts[0].to_uppercase();
        let severity = match candidate.as_str() {
            "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR" | "FATAL" => {
                Severity::parse(&candidate)
            }
            _ => return LogRecord::new(Severity::Info, trimmed),
        };
        LogRecord::new(severity, parts[1])
    } else {
        LogRecord::new(Severity::Info, trimmed)
    }
}

// =============================================================================
// Trait: Source
// =============================================================================

/// A source yields log records. Implementations pull from files, stdin,
/// network sockets, or any other input.
pub trait Source {
    /// Read all available records. Returns an empty vec when exhausted.
    fn read_records(&mut self) -> Vec<LogRecord>;

    /// Human-readable name used for tagging records.
    fn name(&self) -> &str;
}

// =============================================================================
// Trait: Processor
// =============================================================================

/// A processor transforms or filters a batch of log records.
/// Returning an empty vec effectively drops all records.
pub trait Processor {
    fn process(&mut self, records: Vec<LogRecord>) -> Vec<LogRecord>;

    /// Human-readable name for debugging.
    fn name(&self) -> &str;
}

// =============================================================================
// Trait: Destination
// =============================================================================

/// A destination receives processed log records and ships them somewhere.
pub trait Destination {
    fn write_records(&mut self, records: &[LogRecord]) -> io::Result<()>;

    /// Called once when the pipeline shuts down so destinations can flush.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn name(&self) -> &str;
}

// =============================================================================
// Sources
// =============================================================================

/// Reads log lines from an in-memory buffer (simulates file input).
pub struct MemorySource {
    label: String,
    lines: Vec<String>,
    consumed: bool,
}

impl MemorySource {
    pub fn new(label: impl Into<String>, lines: Vec<String>) -> Self {
        Self {
            label: label.into(),
            lines,
            consumed: false,
        }
    }
}

impl Source for MemorySource {
    fn read_records(&mut self) -> Vec<LogRecord> {
        if self.consumed {
            return Vec::new();
        }
        self.consumed = true;
        self.lines
            .iter()
            .map(|l| parse_log_line(l).with_source(self.label.clone()))
            .collect()
    }

    fn name(&self) -> &str {
        &self.label
    }
}

/// Reads log lines from a file path.
pub struct FileSource {
    path: String,
    consumed: bool,
}

impl FileSource {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            consumed: false,
        }
    }
}

impl Source for FileSource {
    fn read_records(&mut self) -> Vec<LogRecord> {
        if self.consumed {
            return Vec::new();
        }
        self.consumed = true;
        let file = match std::fs::File::open(&self.path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("FileSource: failed to open {}: {}", self.path, e);
                return Vec::new();
            }
        };
        io::BufReader::new(file)
            .lines()
            .filter_map(|l| l.ok())
            .map(|l| parse_log_line(&l).with_source(self.path.clone()))
            .collect()
    }

    fn name(&self) -> &str {
        &self.path
    }
}

/// Reads log lines from stdin (one shot, non-blocking with a line limit).
pub struct StdinSource {
    max_lines: usize,
    consumed: bool,
}

impl StdinSource {
    pub fn new(max_lines: usize) -> Self {
        Self {
            max_lines,
            consumed: false,
        }
    }
}

impl Source for StdinSource {
    fn read_records(&mut self) -> Vec<LogRecord> {
        if self.consumed {
            return Vec::new();
        }
        self.consumed = true;
        let stdin = io::stdin();
        stdin
            .lock()
            .lines()
            .take(self.max_lines)
            .filter_map(|l| l.ok())
            .map(|l| parse_log_line(&l).with_source("stdin".to_string()))
            .collect()
    }

    fn name(&self) -> &str {
        "stdin"
    }
}

/// Accepts log lines over a TCP socket (one connection, then done).
pub struct TcpSource {
    addr: String,
    consumed: bool,
}

impl TcpSource {
    pub fn new(addr: impl Into<String>) -> Self {
        Self {
            addr: addr.into(),
            consumed: false,
        }
    }
}

impl Source for TcpSource {
    fn read_records(&mut self) -> Vec<LogRecord> {
        if self.consumed {
            return Vec::new();
        }
        self.consumed = true;
        let listener = match TcpListener::bind(&self.addr) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("TcpSource: bind {} failed: {}", self.addr, e);
                return Vec::new();
            }
        };
        let (stream, _) = match listener.accept() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("TcpSource: accept failed: {}", e);
                return Vec::new();
            }
        };
        io::BufReader::new(stream)
            .lines()
            .filter_map(|l| l.ok())
            .map(|l| parse_log_line(&l).with_source(self.addr.clone()))
            .collect()
    }

    fn name(&self) -> &str {
        &self.addr
    }
}

// =============================================================================
// Processors
// =============================================================================

/// Drops records below a minimum severity threshold.
pub struct SeverityFilter {
    min_severity: Severity,
}

impl SeverityFilter {
    pub fn new(min_severity: Severity) -> Self {
        Self { min_severity }
    }
}

impl Processor for SeverityFilter {
    fn process(&mut self, records: Vec<LogRecord>) -> Vec<LogRecord> {
        records
            .into_iter()
            .filter(|r| r.severity >= self.min_severity)
            .collect()
    }

    fn name(&self) -> &str {
        "severity_filter"
    }
}

/// Replaces values of sensitive fields with `[REDACTED]`.
pub struct FieldRedactor {
    sensitive_keys: Vec<String>,
}

impl FieldRedactor {
    pub fn new(keys: Vec<String>) -> Self {
        Self {
            sensitive_keys: keys,
        }
    }
}

impl Processor for FieldRedactor {
    fn process(&mut self, records: Vec<LogRecord>) -> Vec<LogRecord> {
        records
            .into_iter()
            .map(|mut r| {
                for key in &self.sensitive_keys {
                    if r.fields.contains_key(key) {
                        r.fields.insert(key.clone(), "[REDACTED]".to_string());
                    }
                }
                r
            })
            .collect()
    }

    fn name(&self) -> &str {
        "field_redactor"
    }
}

/// Enriches records that have an `ip` field with mock geo-IP data.
pub struct GeoIpEnricher {
    /// In production this would be a MaxMind DB handle or similar.
    lookup: HashMap<String, String>,
}

impl GeoIpEnricher {
    pub fn new() -> Self {
        // Seed with a few example mappings for demonstration.
        let mut lookup = HashMap::new();
        lookup.insert("192.168.1.1".into(), "US/San Francisco".into());
        lookup.insert("10.0.0.1".into(), "DE/Berlin".into());
        lookup.insert("172.16.0.1".into(), "JP/Tokyo".into());
        Self { lookup }
    }

    pub fn with_mapping(mut self, ip: impl Into<String>, geo: impl Into<String>) -> Self {
        self.lookup.insert(ip.into(), geo.into());
        self
    }
}

impl Processor for GeoIpEnricher {
    fn process(&mut self, records: Vec<LogRecord>) -> Vec<LogRecord> {
        records
            .into_iter()
            .map(|mut r| {
                if let Some(ip) = r.fields.get("ip").cloned() {
                    if let Some(geo) = self.lookup.get(&ip) {
                        r.fields.insert("geo".into(), geo.clone());
                    } else {
                        r.fields.insert("geo".into(), "unknown".into());
                    }
                }
                r
            })
            .collect()
    }

    fn name(&self) -> &str {
        "geo_ip_enricher"
    }
}

/// Tracks counts per severity level. Passes all records through unchanged
/// but accumulates metrics that can be queried after the run.
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

    pub fn counts(&self) -> &HashMap<Severity, u64> {
        &self.counts
    }

    pub fn total(&self) -> u64 {
        self.total
    }
}

impl Processor for MetricsAggregator {
    fn process(&mut self, records: Vec<LogRecord>) -> Vec<LogRecord> {
        for r in &records {
            *self.counts.entry(r.severity).or_insert(0) += 1;
            self.total += 1;
        }
        records
    }

    fn name(&self) -> &str {
        "metrics_aggregator"
    }
}

/// Deterministic sampling: keeps 1 out of every `rate` records.
pub struct Sampler {
    rate: u64,
    counter: u64,
}

impl Sampler {
    /// A rate of 1 keeps everything; a rate of 10 keeps ~10%.
    pub fn new(rate: u64) -> Self {
        assert!(rate >= 1, "sample rate must be >= 1");
        Self { rate, counter: 0 }
    }
}

impl Processor for Sampler {
    fn process(&mut self, records: Vec<LogRecord>) -> Vec<LogRecord> {
        records
            .into_iter()
            .filter(|_| {
                self.counter += 1;
                self.counter % self.rate == 0
            })
            .collect()
    }

    fn name(&self) -> &str {
        "sampler"
    }
}

// =============================================================================
// Destinations
// =============================================================================

/// Prints records to stdout.
pub struct StdoutDestination;

impl Destination for StdoutDestination {
    fn write_records(&mut self, records: &[LogRecord]) -> io::Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        for r in records {
            writeln!(handle, "{}", r)?;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "stdout"
    }
}

/// Appends records to a file.
pub struct FileDestination {
    path: String,
}

impl FileDestination {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

impl Destination for FileDestination {
    fn write_records(&mut self, records: &[LogRecord]) -> io::Result<()> {
        use std::fs::OpenOptions;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        for r in records {
            writeln!(file, "{}", r)?;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        &self.path
    }
}

/// Simulates sending records to Elasticsearch (prints the JSON payload).
pub struct ElasticsearchDestination {
    url: String,
    buffer: Vec<String>,
}

impl ElasticsearchDestination {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            buffer: Vec::new(),
        }
    }

    fn record_to_json(r: &LogRecord) -> String {
        let fields_json: String = r
            .fields
            .iter()
            .map(|(k, v)| format!("\"{}\":\"{}\"", k, v))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"timestamp\":{},\"severity\":\"{}\",\"message\":\"{}\",\"source\":\"{}\",\"fields\":{{{}}}}}",
            r.timestamp, r.severity, r.message, r.source, fields_json
        )
    }
}

impl Destination for ElasticsearchDestination {
    fn write_records(&mut self, records: &[LogRecord]) -> io::Result<()> {
        for r in records {
            self.buffer.push(Self::record_to_json(r));
        }
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            // In production: HTTP POST to self.url with bulk API payload.
            println!(
                "[ES] Would POST {} records to {}",
                self.buffer.len(),
                self.url
            );
            for doc in &self.buffer {
                println!("[ES]   {}", doc);
            }
            self.buffer.clear();
        }
        Ok(())
    }

    fn name(&self) -> &str {
        &self.url
    }
}

/// Simulates pushing records to a Kafka topic.
pub struct KafkaDestination {
    topic: String,
    broker: String,
    sent: u64,
}

impl KafkaDestination {
    pub fn new(broker: impl Into<String>, topic: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            broker: broker.into(),
            sent: 0,
        }
    }

    pub fn sent_count(&self) -> u64 {
        self.sent
    }
}

impl Destination for KafkaDestination {
    fn write_records(&mut self, records: &[LogRecord]) -> io::Result<()> {
        // In production: produce to Kafka via rdkafka or similar.
        for r in records {
            println!(
                "[Kafka] {}:{} <- [{}] {}",
                self.broker, self.topic, r.severity, r.message
            );
            self.sent += 1;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        &self.topic
    }
}

// =============================================================================
// Pipeline: wires sources, processors, and destinations together
// =============================================================================

/// The Pipeline reads from multiple sources, runs records through a chain
/// of processors, and fans out to multiple destinations.
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

    /// Builder method: add a source.
    pub fn source(mut self, s: impl Source + 'static) -> Self {
        self.sources.push(Box::new(s));
        self
    }

    /// Builder method: append a processor to the chain.
    pub fn processor(mut self, p: impl Processor + 'static) -> Self {
        self.processors.push(Box::new(p));
        self
    }

    /// Builder method: add a destination.
    pub fn destination(mut self, d: impl Destination + 'static) -> Self {
        self.destinations.push(Box::new(d));
        self
    }

    /// Execute the pipeline: read -> process -> ship.
    pub fn run(&mut self) -> io::Result<PipelineStats> {
        let mut stats = PipelineStats::default();

        // 1. Collect records from all sources.
        let mut records: Vec<LogRecord> = Vec::new();
        for src in &mut self.sources {
            let batch = src.read_records();
            stats.records_read += batch.len() as u64;
            records.extend(batch);
        }

        // 2. Run through the processor chain.
        for proc in &mut self.processors {
            records = proc.process(records);
        }
        stats.records_after_processing = records.len() as u64;

        // 3. Fan out to all destinations.
        for dest in &mut self.destinations {
            dest.write_records(&records)?;
        }

        // 4. Flush destinations.
        for dest in &mut self.destinations {
            dest.flush()?;
        }

        Ok(stats)
    }
}

/// Summary statistics from a pipeline run.
#[derive(Debug, Default)]
pub struct PipelineStats {
    pub records_read: u64,
    pub records_after_processing: u64,
}

impl fmt::Display for PipelineStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Pipeline stats: {} read, {} after processing",
            self.records_read, self.records_after_processing
        )
    }
}

// =============================================================================
// Demo / main
// =============================================================================

fn main() {
    // Simulate three log sources with in-memory data.
    let app_logs = MemorySource::new(
        "app.log",
        vec![
            "ERROR Failed to connect to database".into(),
            "INFO User login successful".into(),
            "DEBUG Cache hit for key=user:42".into(),
            "WARN Disk usage at 85%".into(),
        ],
    );

    let auth_logs = MemorySource::new(
        "auth.log",
        vec![
            "INFO Authentication attempt from 192.168.1.1".into(),
            "ERROR Invalid token for user admin".into(),
            "FATAL Security breach detected".into(),
        ],
    );

    let network_logs = MemorySource::new(
        "network.log",
        vec![
            "DEBUG Packet received from 10.0.0.1".into(),
            "INFO Connection established to 172.16.0.1".into(),
            "TRACE Heartbeat ping".into(),
            "WARN Connection timeout to 10.0.0.1".into(),
        ],
    );

    // Enrich some records with IP fields for the geo-IP processor.
    // (In production the parser would extract these from the log line.)
    let enriched_source = MemorySource::new(
        "enriched.log",
        vec![
            "INFO Request handled".into(),
        ],
    );

    // Build the processor chain.
    let severity_filter = SeverityFilter::new(Severity::Info); // drop TRACE and DEBUG
    let redactor = FieldRedactor::new(vec!["password".into(), "ssn".into(), "token".into()]);
    let geo_enricher = GeoIpEnricher::new();
    let metrics = MetricsAggregator::new();
    let sampler = Sampler::new(1); // keep everything (rate=1)

    // Build the pipeline.
    let mut pipeline = Pipeline::new()
        // Sources
        .source(app_logs)
        .source(auth_logs)
        .source(network_logs)
        .source(enriched_source)
        // Processors (order matters)
        .processor(severity_filter)
        .processor(redactor)
        .processor(geo_enricher)
        .processor(metrics)
        .processor(sampler)
        // Destinations
        .destination(StdoutDestination)
        .destination(ElasticsearchDestination::new("http://localhost:9200/logs"))
        .destination(KafkaDestination::new("localhost:9092", "logs-topic"));

    println!("=== Running Log Processing Pipeline ===\n");

    match pipeline.run() {
        Ok(stats) => {
            println!("\n=== Pipeline Complete ===");
            println!("{}", stats);
        }
        Err(e) => {
            eprintln!("Pipeline error: {}", e);
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_parse_log_line_with_severity() {
        let r = parse_log_line("ERROR something broke");
        assert_eq!(r.severity, Severity::Error);
        assert_eq!(r.message, "something broke");
    }

    #[test]
    fn test_parse_log_line_default_severity() {
        let r = parse_log_line("just a plain message");
        assert_eq!(r.severity, Severity::Info);
        assert_eq!(r.message, "just a plain message");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Error > Severity::Warn);
        assert!(Severity::Warn > Severity::Info);
        assert!(Severity::Info > Severity::Debug);
        assert!(Severity::Debug > Severity::Trace);
    }

    #[test]
    fn test_severity_filter_drops_below_threshold() {
        let records = vec![
            LogRecord::new(Severity::Debug, "low"),
            LogRecord::new(Severity::Error, "high"),
            LogRecord::new(Severity::Trace, "lowest"),
            LogRecord::new(Severity::Warn, "medium"),
        ];
        let mut filter = SeverityFilter::new(Severity::Warn);
        let result = filter.process(records);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].severity, Severity::Error);
        assert_eq!(result[1].severity, Severity::Warn);
    }

    #[test]
    fn test_field_redactor() {
        let records = vec![
            LogRecord::new(Severity::Info, "login")
                .with_field("password", "secret123")
                .with_field("username", "alice"),
        ];
        let mut redactor = FieldRedactor::new(vec!["password".into()]);
        let result = redactor.process(records);
        assert_eq!(result[0].fields["password"], "[REDACTED]");
        assert_eq!(result[0].fields["username"], "alice");
    }

    #[test]
    fn test_geo_ip_enricher_known_ip() {
        let records = vec![
            LogRecord::new(Severity::Info, "request").with_field("ip", "192.168.1.1"),
        ];
        let mut enricher = GeoIpEnricher::new();
        let result = enricher.process(records);
        assert_eq!(result[0].fields["geo"], "US/San Francisco");
    }

    #[test]
    fn test_geo_ip_enricher_unknown_ip() {
        let records = vec![
            LogRecord::new(Severity::Info, "request").with_field("ip", "8.8.8.8"),
        ];
        let mut enricher = GeoIpEnricher::new();
        let result = enricher.process(records);
        assert_eq!(result[0].fields["geo"], "unknown");
    }

    #[test]
    fn test_geo_ip_enricher_no_ip_field() {
        let records = vec![LogRecord::new(Severity::Info, "no ip here")];
        let mut enricher = GeoIpEnricher::new();
        let result = enricher.process(records);
        assert!(!result[0].fields.contains_key("geo"));
    }

    #[test]
    fn test_metrics_aggregator() {
        let records = vec![
            LogRecord::new(Severity::Info, "a"),
            LogRecord::new(Severity::Info, "b"),
            LogRecord::new(Severity::Error, "c"),
        ];
        let mut metrics = MetricsAggregator::new();
        let result = metrics.process(records);
        assert_eq!(result.len(), 3); // pass-through
        assert_eq!(metrics.total(), 3);
        assert_eq!(*metrics.counts().get(&Severity::Info).unwrap(), 2);
        assert_eq!(*metrics.counts().get(&Severity::Error).unwrap(), 1);
    }

    #[test]
    fn test_sampler_rate_2() {
        let records: Vec<LogRecord> = (0..10)
            .map(|i| LogRecord::new(Severity::Info, format!("msg {}", i)))
            .collect();
        let mut sampler = Sampler::new(2);
        let result = sampler.process(records);
        assert_eq!(result.len(), 5); // keeps every 2nd
    }

    #[test]
    fn test_sampler_rate_1_keeps_all() {
        let records: Vec<LogRecord> = (0..5)
            .map(|i| LogRecord::new(Severity::Info, format!("msg {}", i)))
            .collect();
        let mut sampler = Sampler::new(1);
        let result = sampler.process(records);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_memory_source() {
        let mut src = MemorySource::new("test", vec!["INFO hello".into(), "ERROR boom".into()]);
        let records = src.read_records();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].severity, Severity::Info);
        assert_eq!(records[1].severity, Severity::Error);

        // Second read returns empty (consumed).
        let records2 = src.read_records();
        assert!(records2.is_empty());
    }

    #[test]
    fn test_full_pipeline() {
        let source = MemorySource::new(
            "test",
            vec![
                "DEBUG low".into(),
                "INFO medium".into(),
                "ERROR high".into(),
                "TRACE lowest".into(),
                "WARN warning".into(),
            ],
        );

        // Collect destination: stores records for assertion.
        struct CollectDestination {
            collected: Arc<Mutex<Vec<String>>>,
        }
        impl Destination for CollectDestination {
            fn write_records(&mut self, records: &[LogRecord]) -> io::Result<()> {
                let mut c = self.collected.lock().unwrap();
                for r in records {
                    c.push(format!("{}: {}", r.severity, r.message));
                }
                Ok(())
            }
            fn name(&self) -> &str {
                "collect"
            }
        }

        let collected = Arc::new(Mutex::new(Vec::new()));
        let dest = CollectDestination {
            collected: collected.clone(),
        };

        let mut pipeline = Pipeline::new()
            .source(source)
            .processor(SeverityFilter::new(Severity::Warn))
            .destination(dest);

        let stats = pipeline.run().unwrap();
        assert_eq!(stats.records_read, 5);
        assert_eq!(stats.records_after_processing, 2);

        let results = collected.lock().unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|s| s.contains("ERROR")));
        assert!(results.iter().any(|s| s.contains("WARN")));
    }

    #[test]
    fn test_pipeline_multiple_sources_and_destinations() {
        let src1 = MemorySource::new("s1", vec!["INFO from s1".into()]);
        let src2 = MemorySource::new("s2", vec!["ERROR from s2".into()]);

        struct CountDestination {
            count: Arc<Mutex<u64>>,
        }
        impl Destination for CountDestination {
            fn write_records(&mut self, records: &[LogRecord]) -> io::Result<()> {
                *self.count.lock().unwrap() += records.len() as u64;
                Ok(())
            }
            fn name(&self) -> &str {
                "counter"
            }
        }

        let count1 = Arc::new(Mutex::new(0u64));
        let count2 = Arc::new(Mutex::new(0u64));
        let d1 = CountDestination {
            count: count1.clone(),
        };
        let d2 = CountDestination {
            count: count2.clone(),
        };

        let mut pipeline = Pipeline::new()
            .source(src1)
            .source(src2)
            .destination(d1)
            .destination(d2);

        pipeline.run().unwrap();

        // Both destinations receive all 2 records.
        assert_eq!(*count1.lock().unwrap(), 2);
        assert_eq!(*count2.lock().unwrap(), 2);
    }

    #[test]
    fn test_processor_chain_ordering() {
        // Verify processors run in order: first filter, then the sampler
        // sees only the filtered set.
        let records: Vec<LogRecord> = (0..20)
            .map(|i| {
                if i % 2 == 0 {
                    LogRecord::new(Severity::Error, format!("err {}", i))
                } else {
                    LogRecord::new(Severity::Debug, format!("dbg {}", i))
                }
            })
            .collect();

        let mut src = MemorySource::new("test", Vec::new());
        src.lines = Vec::new(); // will inject records directly

        // Manually run the processor chain.
        let mut filter = SeverityFilter::new(Severity::Warn);
        let mut sampler = Sampler::new(2);

        let after_filter = filter.process(records);
        assert_eq!(after_filter.len(), 10); // only ERROR records

        let after_sample = sampler.process(after_filter);
        assert_eq!(after_sample.len(), 5); // half of 10
    }

    #[test]
    fn test_log_record_display() {
        let r = LogRecord::new(Severity::Error, "disk full")
            .with_field("host", "srv-01");
        let s = format!("{}", r);
        assert!(s.contains("[ERROR]"));
        assert!(s.contains("disk full"));
        assert!(s.contains("host=srv-01"));
    }

    #[test]
    fn test_empty_pipeline() {
        let mut pipeline = Pipeline::new();
        let stats = pipeline.run().unwrap();
        assert_eq!(stats.records_read, 0);
        assert_eq!(stats.records_after_processing, 0);
    }
}
