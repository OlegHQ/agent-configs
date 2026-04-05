//! CI/CD Task Scheduler
//!
//! A task scheduler where tasks have pluggable execution strategies (shell commands,
//! HTTP requests, Docker containers, Rust closures), retry logic, timeouts, dependency
//! tracking, and parallel execution of independent tasks.
//!
//! # Design
//!
//! The **Strategy pattern** drives execution: each `ExecutionStrategy` impl knows how
//! to run one kind of work. The scheduler itself is strategy-agnostic — it only cares
//! about the `ExecutionStrategy` trait, so adding Kubernetes, Lambda, or SSH strategies
//! later requires zero changes to the scheduler.
//!
//! The **dependency graph** is a simple DAG represented via adjacency lists. The
//! scheduler uses a ready-queue approach: tasks whose dependencies are all satisfied
//! are dispatched to a thread pool. When a task completes, its dependents are
//! re-evaluated.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Execution outcome
// ---------------------------------------------------------------------------

/// The result of running a single execution strategy attempt.
#[derive(Debug, Clone)]
pub struct ExecutionOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl ExecutionOutput {
    pub fn success(stdout: impl Into<String>) -> Self {
        Self {
            stdout: stdout.into(),
            stderr: String::new(),
            exit_code: 0,
        }
    }

    pub fn failure(stderr: impl Into<String>, code: i32) -> Self {
        Self {
            stdout: String::new(),
            stderr: stderr.into(),
            exit_code: code,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TaskError {
    ExecutionFailed(String),
    TimedOut(Duration),
    DependencyFailed(String),
    MaxRetriesExceeded { attempts: u32, last_error: String },
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskError::ExecutionFailed(msg) => write!(f, "execution failed: {msg}"),
            TaskError::TimedOut(d) => write!(f, "timed out after {d:?}"),
            TaskError::DependencyFailed(dep) => write!(f, "dependency '{dep}' failed"),
            TaskError::MaxRetriesExceeded {
                attempts,
                last_error,
            } => write!(f, "failed after {attempts} attempts: {last_error}"),
        }
    }
}

pub type TaskResult = Result<ExecutionOutput, TaskError>;

// ---------------------------------------------------------------------------
// Strategy trait  (the core extension point)
// ---------------------------------------------------------------------------

/// Each execution backend implements this trait.  The scheduler calls `execute`
/// and does not care *how* the work is performed — only whether it succeeded.
///
/// `Send + Sync` bounds allow strategies to be shared across threads.
pub trait ExecutionStrategy: Send + Sync {
    /// Human-readable name shown in logs.
    fn name(&self) -> &str;

    /// Run the task.  Implementations should return `Ok(output)` on success
    /// or `Err(TaskError::ExecutionFailed(_))` on failure.
    fn execute(&self) -> TaskResult;
}

// ---------------------------------------------------------------------------
// Concrete strategies
// ---------------------------------------------------------------------------

/// Runs a shell command via `/bin/sh -c`.
pub struct ShellCommand {
    pub command: String,
}

impl ShellCommand {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
        }
    }
}

impl ExecutionStrategy for ShellCommand {
    fn name(&self) -> &str {
        "shell"
    }

    fn execute(&self) -> TaskResult {
        // In a real implementation this would call std::process::Command.
        // For demonstration we simulate success.
        println!("  [shell] running: {}", self.command);
        Ok(ExecutionOutput::success(format!(
            "shell output of `{}`",
            self.command
        )))
    }
}

/// Makes an HTTP request (GET/POST/…).
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub body: Option<String>,
    pub headers: Vec<(String, String)>,
}

impl HttpRequest {
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            method: "GET".into(),
            url: url.into(),
            body: None,
            headers: Vec::new(),
        }
    }

    pub fn post(url: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            method: "POST".into(),
            url: url.into(),
            body: Some(body.into()),
            headers: Vec::new(),
        }
    }
}

impl ExecutionStrategy for HttpRequest {
    fn name(&self) -> &str {
        "http"
    }

    fn execute(&self) -> TaskResult {
        println!("  [http] {} {}", self.method, self.url);
        Ok(ExecutionOutput::success(format!(
            "200 OK from {}",
            self.url
        )))
    }
}

/// Runs a Docker container.
pub struct DockerRun {
    pub image: String,
    pub command: Vec<String>,
    pub env_vars: Vec<(String, String)>,
    pub volumes: Vec<(String, String)>,
}

impl DockerRun {
    pub fn new(image: impl Into<String>, command: Vec<String>) -> Self {
        Self {
            image: image.into(),
            command,
            env_vars: Vec::new(),
            volumes: Vec::new(),
        }
    }
}

impl ExecutionStrategy for DockerRun {
    fn name(&self) -> &str {
        "docker"
    }

    fn execute(&self) -> TaskResult {
        println!(
            "  [docker] running {} with {:?}",
            self.image, self.command
        );
        Ok(ExecutionOutput::success(format!(
            "container {} finished",
            self.image
        )))
    }
}

/// Executes an arbitrary Rust closure.  Useful for in-process validation,
/// artifact manipulation, etc.
pub struct ClosureTask {
    label: String,
    func: Box<dyn Fn() -> TaskResult + Send + Sync>,
}

impl ClosureTask {
    pub fn new(
        label: impl Into<String>,
        func: impl Fn() -> TaskResult + Send + Sync + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            func: Box::new(func),
        }
    }
}

impl ExecutionStrategy for ClosureTask {
    fn name(&self) -> &str {
        "closure"
    }

    fn execute(&self) -> TaskResult {
        println!("  [closure] executing: {}", self.label);
        (self.func)()
    }
}

// ---------------------------------------------------------------------------
// Retry & timeout configuration
// ---------------------------------------------------------------------------

/// Controls how (and whether) a failed task is retried.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of attempts (1 = no retry).
    pub max_attempts: u32,
    /// Delay between retries.
    pub delay: Duration,
    /// Multiplicative factor applied to `delay` after each attempt.
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 1,
            delay: Duration::from_secs(0),
            backoff_multiplier: 1.0,
        }
    }
}

impl RetryPolicy {
    pub fn with_retries(max_attempts: u32, delay: Duration) -> Self {
        Self {
            max_attempts,
            delay,
            backoff_multiplier: 1.0,
        }
    }

    pub fn with_exponential_backoff(
        max_attempts: u32,
        initial_delay: Duration,
        multiplier: f64,
    ) -> Self {
        Self {
            max_attempts,
            delay: initial_delay,
            backoff_multiplier: multiplier,
        }
    }
}

/// Per-task timeout.  `None` means "wait forever".
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    pub duration: Option<Duration>,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self { duration: None }
    }
}

impl TimeoutConfig {
    pub fn seconds(secs: u64) -> Self {
        Self {
            duration: Some(Duration::from_secs(secs)),
        }
    }
}

// ---------------------------------------------------------------------------
// Task definition
// ---------------------------------------------------------------------------

/// A single unit of work in the pipeline.
pub struct Task {
    pub id: String,
    pub strategy: Box<dyn ExecutionStrategy>,
    pub retry_policy: RetryPolicy,
    pub timeout: TimeoutConfig,
    /// IDs of tasks that must complete successfully before this one starts.
    pub dependencies: Vec<String>,
}

impl Task {
    pub fn new(id: impl Into<String>, strategy: impl ExecutionStrategy + 'static) -> Self {
        Self {
            id: id.into(),
            strategy: Box::new(strategy),
            retry_policy: RetryPolicy::default(),
            timeout: TimeoutConfig::default(),
            dependencies: Vec::new(),
        }
    }

    pub fn with_retry(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    pub fn with_timeout(mut self, timeout: TimeoutConfig) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn depends_on(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }
}

// ---------------------------------------------------------------------------
// Scheduler internals
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum TaskStatus {
    Pending,
    Running,
    Succeeded(ExecutionOutput),
    Failed(TaskError),
}

/// Runs a single task, respecting its retry policy and timeout.
fn run_task_with_retries(task: &Task) -> TaskResult {
    let mut last_error = String::new();
    let mut current_delay = task.retry_policy.delay;

    for attempt in 1..=task.retry_policy.max_attempts {
        if attempt > 1 {
            println!(
                "  [retry] task '{}' attempt {}/{}  (delay {:?})",
                task.id, attempt, task.retry_policy.max_attempts, current_delay
            );
            thread::sleep(current_delay);
            current_delay = Duration::from_secs_f64(
                current_delay.as_secs_f64() * task.retry_policy.backoff_multiplier,
            );
        }

        // Timeout enforcement.  In production you would spawn a thread or use
        // async with tokio::time::timeout.  Here we check wall-clock time
        // around the synchronous call (sufficient for the simulated strategies).
        let start = Instant::now();
        let result = task.strategy.execute();
        let elapsed = start.elapsed();

        if let Some(limit) = task.timeout.duration {
            if elapsed > limit {
                return Err(TaskError::TimedOut(limit));
            }
        }

        match result {
            Ok(output) if output.exit_code == 0 => return Ok(output),
            Ok(output) => {
                last_error = format!("non-zero exit code {}: {}", output.exit_code, output.stderr);
            }
            Err(TaskError::ExecutionFailed(msg)) => {
                last_error = msg;
            }
            Err(other) => return Err(other),
        }
    }

    Err(TaskError::MaxRetriesExceeded {
        attempts: task.retry_policy.max_attempts,
        last_error,
    })
}

// ---------------------------------------------------------------------------
// Public scheduler
// ---------------------------------------------------------------------------

/// The DAG-based parallel task scheduler.
///
/// Tasks are submitted via `add_task`, then `run` executes them in dependency
/// order, launching independent tasks on a simple thread pool.
pub struct Scheduler {
    tasks: Vec<Task>,
    max_parallelism: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            max_parallelism: 4,
        }
    }

    pub fn with_parallelism(mut self, n: usize) -> Self {
        self.max_parallelism = n.max(1);
        self
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    /// Validate the task graph (no missing deps, no cycles).
    fn validate(&self) -> Result<(), String> {
        let ids: HashSet<&str> = self.tasks.iter().map(|t| t.id.as_str()).collect();

        // Check for duplicate IDs.
        if ids.len() != self.tasks.len() {
            return Err("duplicate task IDs detected".into());
        }

        // Check that every dependency references an existing task.
        for task in &self.tasks {
            for dep in &task.dependencies {
                if !ids.contains(dep.as_str()) {
                    return Err(format!(
                        "task '{}' depends on unknown task '{dep}'",
                        task.id
                    ));
                }
            }
        }

        // Cycle detection via Kahn's algorithm (topological sort).
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();
        for task in &self.tasks {
            in_degree.entry(task.id.as_str()).or_insert(0);
            for dep in &task.dependencies {
                *in_degree.entry(task.id.as_str()).or_insert(0) += 1;
                dependents
                    .entry(dep.as_str())
                    .or_default()
                    .push(task.id.as_str());
            }
        }

        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut sorted_count = 0usize;
        while let Some(id) = queue.pop_front() {
            sorted_count += 1;
            if let Some(deps) = dependents.get(id) {
                for &d in deps {
                    let deg = in_degree.get_mut(d).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(d);
                    }
                }
            }
        }

        if sorted_count != self.tasks.len() {
            return Err("cycle detected in task dependencies".into());
        }

        Ok(())
    }

    /// Execute all tasks respecting dependencies.  Returns a map of task ID
    /// to final status.
    pub fn run(self) -> Result<HashMap<String, TaskStatus>, String> {
        self.validate()?;

        // Shared state protected by a mutex.  In production you'd use async
        // channels; this keeps the example dependency-free.
        let statuses: Arc<Mutex<HashMap<String, TaskStatus>>> = Arc::new(Mutex::new(
            self.tasks
                .iter()
                .map(|t| (t.id.clone(), TaskStatus::Pending))
                .collect(),
        ));

        // Build index and dependency bookkeeping.
        let task_map: HashMap<String, usize> = self
            .tasks
            .iter()
            .enumerate()
            .map(|(i, t)| (t.id.clone(), i))
            .collect();

        let mut remaining_deps: HashMap<String, HashSet<String>> = self
            .tasks
            .iter()
            .map(|t| {
                (
                    t.id.clone(),
                    t.dependencies.iter().cloned().collect::<HashSet<_>>(),
                )
            })
            .collect();

        // dependents_of[A] = [B, C]  means B and C depend on A.
        let mut dependents_of: HashMap<String, Vec<String>> = HashMap::new();
        for task in &self.tasks {
            for dep in &task.dependencies {
                dependents_of
                    .entry(dep.clone())
                    .or_default()
                    .push(task.id.clone());
            }
        }

        // Move tasks into Arc so threads can borrow them.
        let tasks: Vec<Arc<Task>> = self.tasks.into_iter().map(Arc::new).collect();

        // Ready queue: tasks with zero remaining dependencies.
        let mut ready: VecDeque<String> = remaining_deps
            .iter()
            .filter(|(_, deps)| deps.is_empty())
            .map(|(id, _)| id.clone())
            .collect();

        let mut completed_count = 0usize;
        let total = tasks.len();

        // Simple thread-pool loop.
        while completed_count < total {
            // Launch up to `max_parallelism` ready tasks.
            let mut handles: Vec<(String, thread::JoinHandle<TaskResult>)> = Vec::new();

            while !ready.is_empty() && handles.len() < self.max_parallelism {
                let id = ready.pop_front().unwrap();
                let idx = task_map[&id];
                let task = Arc::clone(&tasks[idx]);

                // Mark running.
                {
                    let mut st = statuses.lock().unwrap();
                    st.insert(id.clone(), TaskStatus::Running);
                }

                println!("[scheduler] starting task '{id}' (strategy: {})", task.strategy.name());
                let handle = thread::spawn(move || run_task_with_retries(&task));
                handles.push((id, handle));
            }

            if handles.is_empty() {
                // Nothing is ready and nothing is running — should not happen
                // after validation, but guard against it.
                return Err("deadlock: no runnable tasks remain".into());
            }

            // Wait for the current batch to finish.
            for (id, handle) in handles {
                let result = handle.join().expect("task thread panicked");

                let mut st = statuses.lock().unwrap();
                match &result {
                    Ok(output) => {
                        println!("[scheduler] task '{id}' succeeded");
                        st.insert(id.clone(), TaskStatus::Succeeded(output.clone()));

                        // Unblock dependents.
                        if let Some(deps) = dependents_of.get(&id) {
                            for dep_id in deps {
                                if let Some(rem) = remaining_deps.get_mut(dep_id) {
                                    rem.remove(&id);
                                    if rem.is_empty() {
                                        ready.push_back(dep_id.clone());
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => {
                        println!("[scheduler] task '{id}' failed: {err}");
                        st.insert(id.clone(), TaskStatus::Failed(err.clone()));

                        // Cascade failure to all transitive dependents.
                        let mut cascade: VecDeque<String> = VecDeque::new();
                        if let Some(deps) = dependents_of.get(&id) {
                            cascade.extend(deps.iter().cloned());
                        }
                        while let Some(dep_id) = cascade.pop_front() {
                            if matches!(st.get(&dep_id), Some(TaskStatus::Pending)) {
                                st.insert(
                                    dep_id.clone(),
                                    TaskStatus::Failed(TaskError::DependencyFailed(id.clone())),
                                );
                                // Remove from remaining so it's never queued.
                                remaining_deps.remove(&dep_id);
                                if let Some(transitive) = dependents_of.get(&dep_id) {
                                    cascade.extend(transitive.iter().cloned());
                                }
                            }
                        }
                    }
                }

                completed_count += 1;
            }
        }

        Ok(Arc::try_unwrap(statuses).unwrap().into_inner().unwrap())
    }
}

// ---------------------------------------------------------------------------
// Demo / tests
// ---------------------------------------------------------------------------

fn main() {
    println!("=== CI/CD Task Scheduler Demo ===\n");

    let mut scheduler = Scheduler::new().with_parallelism(4);

    // Stage 1: independent tasks (will run in parallel)
    scheduler.add_task(
        Task::new("checkout", ShellCommand::new("git clone https://github.com/example/repo"))
            .with_timeout(TimeoutConfig::seconds(60)),
    );

    scheduler.add_task(
        Task::new(
            "pull-base-image",
            DockerRun::new("alpine:3.18".to_string(), vec!["echo".into(), "ready".into()]),
        )
        .with_retry(RetryPolicy::with_retries(3, Duration::from_secs(2))),
    );

    // Stage 2: depends on checkout
    scheduler.add_task(
        Task::new("lint", ShellCommand::new("cargo clippy -- -D warnings"))
            .depends_on("checkout"),
    );

    scheduler.add_task(
        Task::new("test", ShellCommand::new("cargo test --all"))
            .depends_on("checkout")
            .with_retry(RetryPolicy::with_retries(2, Duration::from_secs(5)))
            .with_timeout(TimeoutConfig::seconds(300)),
    );

    // Stage 3: depends on lint + test + base image
    scheduler.add_task(
        Task::new(
            "build-image",
            DockerRun::new("docker:24".to_string(), vec!["docker".into(), "build".into(), ".".into()]),
        )
        .depends_on("lint")
        .depends_on("test")
        .depends_on("pull-base-image"),
    );

    // Stage 4: deploy — HTTP call that depends on image build
    scheduler.add_task(
        Task::new(
            "deploy",
            HttpRequest::post("https://deploy.example.com/api/release", r#"{"tag":"latest"}"#),
        )
        .depends_on("build-image")
        .with_retry(RetryPolicy::with_exponential_backoff(
            3,
            Duration::from_secs(1),
            2.0,
        )),
    );

    // Stage 4 (parallel with deploy): run a closure to generate a report
    scheduler.add_task(
        Task::new(
            "report",
            ClosureTask::new("generate deployment report", || {
                Ok(ExecutionOutput::success("report generated"))
            }),
        )
        .depends_on("build-image"),
    );

    // Run the pipeline.
    match scheduler.run() {
        Ok(results) => {
            println!("\n=== Final Results ===");
            for (id, status) in &results {
                let summary = match status {
                    TaskStatus::Pending => "pending".to_string(),
                    TaskStatus::Running => "running".to_string(),
                    TaskStatus::Succeeded(_) => "succeeded".to_string(),
                    TaskStatus::Failed(e) => format!("FAILED: {e}"),
                };
                println!("  {id}: {summary}");
            }
        }
        Err(e) => eprintln!("Scheduler error: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper strategy that always succeeds.
    struct AlwaysOk;
    impl ExecutionStrategy for AlwaysOk {
        fn name(&self) -> &str { "ok" }
        fn execute(&self) -> TaskResult { Ok(ExecutionOutput::success("ok")) }
    }

    /// Helper strategy that always fails.
    struct AlwaysFail;
    impl ExecutionStrategy for AlwaysFail {
        fn name(&self) -> &str { "fail" }
        fn execute(&self) -> TaskResult {
            Err(TaskError::ExecutionFailed("boom".into()))
        }
    }

    #[test]
    fn single_task_succeeds() {
        let mut s = Scheduler::new();
        s.add_task(Task::new("a", AlwaysOk));
        let results = s.run().unwrap();
        assert!(matches!(results["a"], TaskStatus::Succeeded(_)));
    }

    #[test]
    fn dependency_chain() {
        let mut s = Scheduler::new();
        s.add_task(Task::new("a", AlwaysOk));
        s.add_task(Task::new("b", AlwaysOk).depends_on("a"));
        s.add_task(Task::new("c", AlwaysOk).depends_on("b"));
        let results = s.run().unwrap();
        assert!(matches!(results["c"], TaskStatus::Succeeded(_)));
    }

    #[test]
    fn failure_cascades() {
        let mut s = Scheduler::new();
        s.add_task(Task::new("a", AlwaysFail));
        s.add_task(Task::new("b", AlwaysOk).depends_on("a"));
        let results = s.run().unwrap();
        assert!(matches!(results["a"], TaskStatus::Failed(_)));
        assert!(matches!(results["b"], TaskStatus::Failed(TaskError::DependencyFailed(_))));
    }

    #[test]
    fn cycle_detection() {
        let mut s = Scheduler::new();
        s.add_task(Task::new("a", AlwaysOk).depends_on("b"));
        s.add_task(Task::new("b", AlwaysOk).depends_on("a"));
        let err = s.run().unwrap_err();
        assert!(err.contains("cycle"));
    }

    #[test]
    fn missing_dependency_detected() {
        let mut s = Scheduler::new();
        s.add_task(Task::new("a", AlwaysOk).depends_on("nonexistent"));
        let err = s.run().unwrap_err();
        assert!(err.contains("unknown task"));
    }

    #[test]
    fn retry_eventually_succeeds() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        let mut s = Scheduler::new();
        s.add_task(
            Task::new(
                "flaky",
                ClosureTask::new("flaky task", move || {
                    let n = c.fetch_add(1, Ordering::SeqCst);
                    if n < 2 {
                        Err(TaskError::ExecutionFailed("not yet".into()))
                    } else {
                        Ok(ExecutionOutput::success("finally"))
                    }
                }),
            )
            .with_retry(RetryPolicy::with_retries(3, Duration::from_millis(10))),
        );

        let results = s.run().unwrap();
        assert!(matches!(results["flaky"], TaskStatus::Succeeded(_)));
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn parallel_independent_tasks() {
        // Four independent tasks should all complete.
        let mut s = Scheduler::new().with_parallelism(4);
        for i in 0..4 {
            s.add_task(Task::new(format!("t{i}"), AlwaysOk));
        }
        let results = s.run().unwrap();
        assert_eq!(results.len(), 4);
        for (_, status) in &results {
            assert!(matches!(status, TaskStatus::Succeeded(_)));
        }
    }

    #[test]
    fn different_strategies_compose() {
        let mut s = Scheduler::new();
        s.add_task(Task::new("shell", ShellCommand::new("echo hi")));
        s.add_task(Task::new("http", HttpRequest::get("https://example.com")).depends_on("shell"));
        s.add_task(
            Task::new(
                "docker",
                DockerRun::new("alpine", vec!["echo".into(), "hi".into()]),
            )
            .depends_on("shell"),
        );
        s.add_task(
            Task::new(
                "closure",
                ClosureTask::new("finalize", || Ok(ExecutionOutput::success("done"))),
            )
            .depends_on("http")
            .depends_on("docker"),
        );

        let results = s.run().unwrap();
        assert!(matches!(results["closure"], TaskStatus::Succeeded(_)));
    }
}
