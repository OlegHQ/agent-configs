// CI/CD Task Scheduler
//
// Design patterns applied:
//   - **Strategy** (TaskExecutor trait): open set of execution strategies.
//     Adding Kubernetes, Lambda, or SSH = one new struct + impl. No existing code touched.
//   - **Decorator** (RetryExecutor): wraps any executor to add retry logic transparently.
//   - **Builder** (TaskDefinitionBuilder): staged construction of task definitions.
//   - Enum for **closed** sets: TaskStatus (fixed lifecycle states).
//
// Change vectors addressed:
//   1. New execution strategies  -> implement TaskExecutor
//   2. Cross-cutting concerns    -> Decorator wrappers (retry, timeout)
//   3. Scheduling policy         -> Scheduler owns the DAG; parallelism via async tasks

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Core result / error types
// ---------------------------------------------------------------------------

/// Unified error type for task execution.
#[derive(Debug, Clone)]
pub struct TaskError {
    pub message: String,
    pub retryable: bool,
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TaskError {}

pub type TaskResult = Result<TaskOutput, TaskError>;

/// Output produced by a successful task execution.
#[derive(Debug, Clone)]
pub struct TaskOutput {
    pub stdout: String,
    pub exit_code: i32,
}

// ---------------------------------------------------------------------------
// TaskStatus — closed enum (fixed lifecycle states, exhaustive matching)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Waiting,   // blocked on dependencies
    Running,
    Succeeded(TaskOutput),
    Failed(TaskError),
    TimedOut,
    Cancelled,
}

impl TaskStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskStatus::Succeeded(_) | TaskStatus::Failed(_) | TaskStatus::TimedOut | TaskStatus::Cancelled
        )
    }

    pub fn is_success(&self) -> bool {
        matches!(self, TaskStatus::Succeeded(_))
    }
}

// ---------------------------------------------------------------------------
// Strategy trait: TaskExecutor — open set of execution strategies
// ---------------------------------------------------------------------------

/// Each execution strategy implements this trait.
/// Adding a new strategy (K8s, Lambda, SSH) = a new struct + impl.
/// No existing code is modified.
pub trait TaskExecutor: Send + Sync {
    /// Human-readable name of this strategy (for logging / UI).
    fn strategy_name(&self) -> &str;

    /// Execute the task asynchronously.
    fn execute(&self) -> Pin<Box<dyn Future<Output = TaskResult> + Send + '_>>;
}

// ---------------------------------------------------------------------------
// Concrete strategies
// ---------------------------------------------------------------------------

/// Runs a shell command via `sh -c`.
pub struct ShellExecutor {
    pub command: String,
    pub env: HashMap<String, String>,
    pub working_dir: Option<String>,
}

impl TaskExecutor for ShellExecutor {
    fn strategy_name(&self) -> &str {
        "shell"
    }

    fn execute(&self) -> Pin<Box<dyn Future<Output = TaskResult> + Send + '_>> {
        Box::pin(async move {
            let mut cmd = tokio::process::Command::new("sh");
            cmd.arg("-c").arg(&self.command);
            if let Some(ref dir) = self.working_dir {
                cmd.current_dir(dir);
            }
            for (k, v) in &self.env {
                cmd.env(k, v);
            }
            let output = cmd
                .output()
                .await
                .map_err(|e| TaskError {
                    message: format!("Failed to spawn shell: {e}"),
                    retryable: true,
                })?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let code = output.status.code().unwrap_or(-1);
            if output.status.success() {
                Ok(TaskOutput {
                    stdout,
                    exit_code: code,
                })
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(TaskError {
                    message: format!("Shell exited {code}: {stderr}"),
                    retryable: false,
                })
            }
        })
    }
}

/// Makes an HTTP request.
pub struct HttpExecutor {
    pub method: HttpMethod,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub expected_status: u16,
}

#[derive(Debug, Clone)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl TaskExecutor for HttpExecutor {
    fn strategy_name(&self) -> &str {
        "http"
    }

    fn execute(&self) -> Pin<Box<dyn Future<Output = TaskResult> + Send + '_>> {
        Box::pin(async move {
            // Real implementation would use reqwest or hyper.
            // Stubbed for compilation without heavyweight dependencies.
            let _ = (&self.method, &self.url, &self.headers, &self.body);
            Ok(TaskOutput {
                stdout: format!("HTTP {} {} -> {}", format!("{:?}", self.method), self.url, self.expected_status),
                exit_code: 0,
            })
        })
    }
}

/// Runs a Docker container.
pub struct DockerExecutor {
    pub image: String,
    pub command: Vec<String>,
    pub env: HashMap<String, String>,
    pub volumes: Vec<(String, String)>, // host:container
    pub remove_after: bool,
}

impl TaskExecutor for DockerExecutor {
    fn strategy_name(&self) -> &str {
        "docker"
    }

    fn execute(&self) -> Pin<Box<dyn Future<Output = TaskResult> + Send + '_>> {
        Box::pin(async move {
            let mut args = vec!["run".to_string()];
            if self.remove_after {
                args.push("--rm".to_string());
            }
            for (k, v) in &self.env {
                args.push("-e".to_string());
                args.push(format!("{k}={v}"));
            }
            for (host, container) in &self.volumes {
                args.push("-v".to_string());
                args.push(format!("{host}:{container}"));
            }
            args.push(self.image.clone());
            args.extend(self.command.iter().cloned());

            let output = tokio::process::Command::new("docker")
                .args(&args)
                .output()
                .await
                .map_err(|e| TaskError {
                    message: format!("Failed to run docker: {e}"),
                    retryable: true,
                })?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let code = output.status.code().unwrap_or(-1);
            if output.status.success() {
                Ok(TaskOutput {
                    stdout,
                    exit_code: code,
                })
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(TaskError {
                    message: format!("Docker exited {code}: {stderr}"),
                    retryable: false,
                })
            }
        })
    }
}

/// Executes an arbitrary async Rust closure.
/// Useful for in-process logic (validation, artifact manipulation, etc.).
pub struct ClosureExecutor<F>
where
    F: Fn() -> Pin<Box<dyn Future<Output = TaskResult> + Send>> + Send + Sync,
{
    pub name: String,
    pub func: F,
}

impl<F> TaskExecutor for ClosureExecutor<F>
where
    F: Fn() -> Pin<Box<dyn Future<Output = TaskResult> + Send>> + Send + Sync,
{
    fn strategy_name(&self) -> &str {
        &self.name
    }

    fn execute(&self) -> Pin<Box<dyn Future<Output = TaskResult> + Send + '_>> {
        (self.func)()
    }
}

// ---------------------------------------------------------------------------
// Decorator: RetryExecutor — wraps any executor to add retry logic
// ---------------------------------------------------------------------------

/// Wraps any `TaskExecutor` and retries on retryable failures.
/// Pattern: **Decorator** — same trait, delegates + enhances.
pub struct RetryExecutor {
    pub inner: Box<dyn TaskExecutor>,
    pub max_retries: u32,
    pub backoff: Duration,
}

impl TaskExecutor for RetryExecutor {
    fn strategy_name(&self) -> &str {
        self.inner.strategy_name()
    }

    fn execute(&self) -> Pin<Box<dyn Future<Output = TaskResult> + Send + '_>> {
        Box::pin(async move {
            let mut last_err = None;
            for attempt in 0..=self.max_retries {
                match self.inner.execute().await {
                    Ok(output) => return Ok(output),
                    Err(e) if e.retryable && attempt < self.max_retries => {
                        eprintln!(
                            "[retry] {} attempt {}/{} failed: {}",
                            self.inner.strategy_name(),
                            attempt + 1,
                            self.max_retries,
                            e.message
                        );
                        last_err = Some(e);
                        tokio::time::sleep(self.backoff * (attempt + 1)).await;
                    }
                    Err(e) => return Err(e),
                }
            }
            Err(last_err.unwrap_or_else(|| TaskError {
                message: "Exhausted retries with no result".into(),
                retryable: false,
            }))
        })
    }
}

// ---------------------------------------------------------------------------
// TaskDefinition — the unit of work scheduled by the scheduler
// ---------------------------------------------------------------------------

pub type TaskId = String;

pub struct TaskDefinition {
    pub id: TaskId,
    pub executor: Box<dyn TaskExecutor>,
    pub timeout: Option<Duration>,
    pub dependencies: Vec<TaskId>,
}

// ---------------------------------------------------------------------------
// Builder for TaskDefinition
// ---------------------------------------------------------------------------

pub struct TaskDefinitionBuilder {
    id: TaskId,
    executor: Option<Box<dyn TaskExecutor>>,
    timeout: Option<Duration>,
    dependencies: Vec<TaskId>,
    max_retries: Option<u32>,
    retry_backoff: Option<Duration>,
}

impl TaskDefinitionBuilder {
    pub fn new(id: impl Into<TaskId>) -> Self {
        Self {
            id: id.into(),
            executor: None,
            timeout: None,
            dependencies: Vec::new(),
            max_retries: None,
            retry_backoff: None,
        }
    }

    pub fn executor(mut self, exec: impl TaskExecutor + 'static) -> Self {
        self.executor = Some(Box::new(exec));
        self
    }

    pub fn timeout(mut self, dur: Duration) -> Self {
        self.timeout = Some(dur);
        self
    }

    pub fn depends_on(mut self, dep: impl Into<TaskId>) -> Self {
        self.dependencies.push(dep.into());
        self
    }

    pub fn retry(mut self, max_retries: u32, backoff: Duration) -> Self {
        self.max_retries = Some(max_retries);
        self.retry_backoff = Some(backoff);
        self
    }

    pub fn build(self) -> TaskDefinition {
        let mut executor = self.executor.expect("executor is required");

        // Wrap with retry decorator if configured.
        if let Some(retries) = self.max_retries {
            executor = Box::new(RetryExecutor {
                inner: executor,
                max_retries: retries,
                backoff: self.retry_backoff.unwrap_or(Duration::from_secs(1)),
            });
        }

        TaskDefinition {
            id: self.id,
            executor,
            timeout: self.timeout,
            dependencies: self.dependencies,
        }
    }
}

// ---------------------------------------------------------------------------
// Scheduler — runs tasks respecting dependencies, parallelizing independent work
// ---------------------------------------------------------------------------

/// Result of a complete scheduler run.
#[derive(Debug)]
pub struct SchedulerReport {
    pub statuses: HashMap<TaskId, TaskStatus>,
}

impl SchedulerReport {
    pub fn all_succeeded(&self) -> bool {
        self.statuses.values().all(|s| s.is_success())
    }

    pub fn failed_tasks(&self) -> Vec<&TaskId> {
        self.statuses
            .iter()
            .filter(|(_, s)| matches!(s, TaskStatus::Failed(_) | TaskStatus::TimedOut))
            .map(|(id, _)| id)
            .collect()
    }
}

pub struct Scheduler {
    tasks: Vec<TaskDefinition>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn add_task(&mut self, task: TaskDefinition) {
        self.tasks.push(task);
    }

    /// Validate the task graph: check for missing dependencies and cycles.
    fn validate(&self) -> Result<(), String> {
        let ids: HashSet<&str> = self.tasks.iter().map(|t| t.id.as_str()).collect();

        // Check for missing dependencies.
        for task in &self.tasks {
            for dep in &task.dependencies {
                if !ids.contains(dep.as_str()) {
                    return Err(format!(
                        "Task '{}' depends on '{}', which is not registered",
                        task.id, dep
                    ));
                }
            }
        }

        // Check for cycles via topological sort (Kahn's algorithm).
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
        for task in &self.tasks {
            in_degree.entry(task.id.as_str()).or_insert(0);
            for dep in &task.dependencies {
                adjacency.entry(dep.as_str()).or_default().push(task.id.as_str());
                *in_degree.entry(task.id.as_str()).or_insert(0) += 1;
            }
        }

        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut visited = 0usize;
        while let Some(node) = queue.pop_front() {
            visited += 1;
            if let Some(dependents) = adjacency.get(node) {
                for &dep in dependents {
                    let deg = in_degree.get_mut(dep).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(dep);
                    }
                }
            }
        }

        if visited != self.tasks.len() {
            return Err("Dependency cycle detected in task graph".into());
        }

        Ok(())
    }

    /// Run all tasks, respecting dependencies and parallelizing where possible.
    pub async fn run(self) -> SchedulerReport {
        if let Err(e) = self.validate() {
            panic!("Scheduler validation failed: {e}");
        }

        let statuses: Arc<tokio::sync::RwLock<HashMap<TaskId, TaskStatus>>> =
            Arc::new(tokio::sync::RwLock::new(HashMap::new()));

        // Initialize all tasks as Pending.
        {
            let mut s = statuses.write().await;
            for task in &self.tasks {
                s.insert(task.id.clone(), TaskStatus::Pending);
            }
        }

        // Move tasks into an Arc-wrapped map so we can hand them off to spawned futures.
        let task_map: HashMap<TaskId, TaskDefinition> = self
            .tasks
            .into_iter()
            .map(|t| (t.id.clone(), t))
            .collect();
        let task_map = Arc::new(tokio::sync::Mutex::new(task_map));

        // Use Notify channels: one per task, signalled on completion.
        let notifiers: HashMap<TaskId, Arc<tokio::sync::Notify>> = {
            let tm = task_map.lock().await;
            tm.keys()
                .map(|id| (id.clone(), Arc::new(tokio::sync::Notify::new())))
                .collect()
        };
        let notifiers = Arc::new(notifiers);

        // Spawn a future for each task.
        let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::new();

        let all_ids: Vec<TaskId> = {
            let tm = task_map.lock().await;
            tm.keys().cloned().collect()
        };

        for task_id in all_ids {
            let statuses = Arc::clone(&statuses);
            let task_map = Arc::clone(&task_map);
            let notifiers = Arc::clone(&notifiers);
            let id = task_id.clone();

            handles.push(tokio::spawn(async move {
                // Gather dependency info before taking the task out of the map.
                let deps: Vec<TaskId> = {
                    let tm = task_map.lock().await;
                    tm.get(&id).map(|t| t.dependencies.clone()).unwrap_or_default()
                };

                // Wait for all dependencies.
                {
                    let mut s = statuses.write().await;
                    if !deps.is_empty() {
                        s.insert(id.clone(), TaskStatus::Waiting);
                    }
                }

                for dep_id in &deps {
                    // Wait until notified.
                    loop {
                        {
                            let s = statuses.read().await;
                            if let Some(status) = s.get(dep_id.as_str()) {
                                if status.is_terminal() {
                                    break;
                                }
                            }
                        }
                        if let Some(n) = notifiers.get(dep_id) {
                            n.notified().await;
                        }
                    }

                    // If a dependency failed, cancel this task.
                    {
                        let s = statuses.read().await;
                        if let Some(dep_status) = s.get(dep_id.as_str()) {
                            if !dep_status.is_success() {
                                let mut sw = statuses.write().await;
                                sw.insert(id.clone(), TaskStatus::Cancelled);
                                // Notify anyone waiting on us.
                                if let Some(n) = notifiers.get(&id) {
                                    n.notify_waiters();
                                }
                                return;
                            }
                        }
                    }
                }

                // Take ownership of the task.
                let task = {
                    let mut tm = task_map.lock().await;
                    tm.remove(&id)
                };
                let task = match task {
                    Some(t) => t,
                    None => return,
                };

                // Mark running.
                {
                    let mut s = statuses.write().await;
                    s.insert(id.clone(), TaskStatus::Running);
                }

                // Execute with optional timeout.
                let result = if let Some(timeout_dur) = task.timeout {
                    match tokio::time::timeout(timeout_dur, task.executor.execute()).await {
                        Ok(res) => res,
                        Err(_) => {
                            let mut s = statuses.write().await;
                            s.insert(id.clone(), TaskStatus::TimedOut);
                            if let Some(n) = notifiers.get(&id) {
                                n.notify_waiters();
                            }
                            return;
                        }
                    }
                } else {
                    task.executor.execute().await
                };

                // Record result.
                {
                    let mut s = statuses.write().await;
                    match result {
                        Ok(output) => s.insert(id.clone(), TaskStatus::Succeeded(output)),
                        Err(e) => s.insert(id.clone(), TaskStatus::Failed(e)),
                    };
                }

                // Notify dependents.
                if let Some(n) = notifiers.get(&id) {
                    n.notify_waiters();
                }
            }));
        }

        // Await all tasks.
        for handle in handles {
            let _ = handle.await;
        }

        let statuses = Arc::try_unwrap(statuses)
            .expect("all handles joined")
            .into_inner();

        SchedulerReport { statuses }
    }
}

// ---------------------------------------------------------------------------
// Example / smoke test (runs with `cargo test` or as a binary)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a closure executor that succeeds immediately.
    fn ok_executor(label: &str) -> ClosureExecutor<impl Fn() -> Pin<Box<dyn Future<Output = TaskResult> + Send>> + Send + Sync>
    {
        let label = label.to_string();
        ClosureExecutor {
            name: label.clone(),
            func: move || {
                let l = label.clone();
                Box::pin(async move {
                    Ok(TaskOutput {
                        stdout: format!("{l} done"),
                        exit_code: 0,
                    })
                })
            },
        }
    }

    /// Helper: build a closure executor that fails.
    fn failing_executor(label: &str) -> ClosureExecutor<impl Fn() -> Pin<Box<dyn Future<Output = TaskResult> + Send>> + Send + Sync>
    {
        let label = label.to_string();
        ClosureExecutor {
            name: label.clone(),
            func: move || {
                let l = label.clone();
                Box::pin(async move {
                    Err(TaskError {
                        message: format!("{l} failed"),
                        retryable: false,
                    })
                })
            },
        }
    }

    #[tokio::test]
    async fn independent_tasks_run_in_parallel() {
        let mut scheduler = Scheduler::new();
        scheduler.add_task(
            TaskDefinitionBuilder::new("a")
                .executor(ok_executor("a"))
                .build(),
        );
        scheduler.add_task(
            TaskDefinitionBuilder::new("b")
                .executor(ok_executor("b"))
                .build(),
        );
        scheduler.add_task(
            TaskDefinitionBuilder::new("c")
                .executor(ok_executor("c"))
                .build(),
        );

        let report = scheduler.run().await;
        assert!(report.all_succeeded());
        assert_eq!(report.statuses.len(), 3);
    }

    #[tokio::test]
    async fn dependencies_are_respected() {
        let mut scheduler = Scheduler::new();

        // a -> b -> c (sequential chain)
        scheduler.add_task(
            TaskDefinitionBuilder::new("a")
                .executor(ok_executor("a"))
                .build(),
        );
        scheduler.add_task(
            TaskDefinitionBuilder::new("b")
                .executor(ok_executor("b"))
                .depends_on("a")
                .build(),
        );
        scheduler.add_task(
            TaskDefinitionBuilder::new("c")
                .executor(ok_executor("c"))
                .depends_on("b")
                .build(),
        );

        let report = scheduler.run().await;
        assert!(report.all_succeeded());
    }

    #[tokio::test]
    async fn failed_dependency_cancels_downstream() {
        let mut scheduler = Scheduler::new();

        scheduler.add_task(
            TaskDefinitionBuilder::new("a")
                .executor(failing_executor("a"))
                .build(),
        );
        scheduler.add_task(
            TaskDefinitionBuilder::new("b")
                .executor(ok_executor("b"))
                .depends_on("a")
                .build(),
        );

        let report = scheduler.run().await;
        assert!(!report.all_succeeded());
        assert!(matches!(report.statuses.get("a"), Some(TaskStatus::Failed(_))));
        assert!(matches!(report.statuses.get("b"), Some(TaskStatus::Cancelled)));
    }

    #[tokio::test]
    async fn timeout_triggers_timedout_status() {
        let mut scheduler = Scheduler::new();

        scheduler.add_task(
            TaskDefinitionBuilder::new("slow")
                .executor(ClosureExecutor {
                    name: "slow".to_string(),
                    func: || {
                        Box::pin(async {
                            tokio::time::sleep(Duration::from_secs(10)).await;
                            Ok(TaskOutput {
                                stdout: "done".into(),
                                exit_code: 0,
                            })
                        })
                    },
                })
                .timeout(Duration::from_millis(50))
                .build(),
        );

        let report = scheduler.run().await;
        assert!(matches!(report.statuses.get("slow"), Some(TaskStatus::TimedOut)));
    }

    #[tokio::test]
    async fn diamond_dependency_graph() {
        //     a
        //    / \
        //   b   c
        //    \ /
        //     d
        let mut scheduler = Scheduler::new();
        scheduler.add_task(TaskDefinitionBuilder::new("a").executor(ok_executor("a")).build());
        scheduler.add_task(TaskDefinitionBuilder::new("b").executor(ok_executor("b")).depends_on("a").build());
        scheduler.add_task(TaskDefinitionBuilder::new("c").executor(ok_executor("c")).depends_on("a").build());
        scheduler.add_task(
            TaskDefinitionBuilder::new("d")
                .executor(ok_executor("d"))
                .depends_on("b")
                .depends_on("c")
                .build(),
        );

        let report = scheduler.run().await;
        assert!(report.all_succeeded());
    }

    #[tokio::test]
    #[should_panic(expected = "cycle detected")]
    async fn cycle_detection() {
        let mut scheduler = Scheduler::new();
        scheduler.add_task(TaskDefinitionBuilder::new("a").executor(ok_executor("a")).depends_on("b").build());
        scheduler.add_task(TaskDefinitionBuilder::new("b").executor(ok_executor("b")).depends_on("a").build());
        scheduler.run().await;
    }

    #[tokio::test]
    async fn retry_decorator_retries_on_retryable_error() {
        use std::sync::atomic::{AtomicU32, Ordering};

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);

        let exec = ClosureExecutor {
            name: "flaky".to_string(),
            func: move || {
                let c = Arc::clone(&counter_clone);
                Box::pin(async move {
                    let attempt = c.fetch_add(1, Ordering::SeqCst);
                    if attempt < 2 {
                        Err(TaskError {
                            message: "transient".into(),
                            retryable: true,
                        })
                    } else {
                        Ok(TaskOutput {
                            stdout: "recovered".into(),
                            exit_code: 0,
                        })
                    }
                })
            },
        };

        let mut scheduler = Scheduler::new();
        scheduler.add_task(
            TaskDefinitionBuilder::new("flaky")
                .executor(exec)
                .retry(3, Duration::from_millis(10))
                .build(),
        );

        let report = scheduler.run().await;
        assert!(report.all_succeeded());
        assert_eq!(counter.load(Ordering::SeqCst), 3); // 2 failures + 1 success
    }
}

// ---------------------------------------------------------------------------
// Main — demonstration entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    println!("=== CI/CD Task Scheduler Demo ===\n");

    let mut scheduler = Scheduler::new();

    // Stage 1: Checkout (shell)
    scheduler.add_task(
        TaskDefinitionBuilder::new("checkout")
            .executor(ShellExecutor {
                command: "echo 'Cloning repo...'".into(),
                env: HashMap::new(),
                working_dir: None,
            })
            .timeout(Duration::from_secs(60))
            .build(),
    );

    // Stage 2: Lint + Test in parallel (both depend on checkout)
    scheduler.add_task(
        TaskDefinitionBuilder::new("lint")
            .executor(ShellExecutor {
                command: "echo 'Running clippy...'".into(),
                env: HashMap::new(),
                working_dir: None,
            })
            .depends_on("checkout")
            .timeout(Duration::from_secs(120))
            .build(),
    );

    scheduler.add_task(
        TaskDefinitionBuilder::new("test")
            .executor(DockerExecutor {
                image: "rust:latest".into(),
                command: vec!["cargo".into(), "test".into()],
                env: HashMap::new(),
                volumes: vec![],
                remove_after: true,
            })
            .depends_on("checkout")
            .timeout(Duration::from_secs(300))
            .retry(2, Duration::from_secs(5))
            .build(),
    );

    // Stage 3: Deploy (depends on both lint and test)
    scheduler.add_task(
        TaskDefinitionBuilder::new("deploy")
            .executor(HttpExecutor {
                method: HttpMethod::Post,
                url: "https://deploy.example.com/api/deploy".into(),
                headers: HashMap::new(),
                body: Some(r#"{"version": "1.0.0"}"#.into()),
                expected_status: 200,
            })
            .depends_on("lint")
            .depends_on("test")
            .build(),
    );

    // Stage 4: Notify (closure, depends on deploy)
    scheduler.add_task(
        TaskDefinitionBuilder::new("notify")
            .executor(ClosureExecutor {
                name: "slack-notify".into(),
                func: || {
                    Box::pin(async {
                        println!("  Sending Slack notification...");
                        Ok(TaskOutput {
                            stdout: "Notification sent".into(),
                            exit_code: 0,
                        })
                    })
                },
            })
            .depends_on("deploy")
            .build(),
    );

    let report = scheduler.run().await;

    println!("\n=== Results ===");
    for (id, status) in &report.statuses {
        let symbol = match status {
            TaskStatus::Succeeded(_) => "[OK]",
            TaskStatus::Failed(_) => "[FAIL]",
            TaskStatus::TimedOut => "[TIMEOUT]",
            TaskStatus::Cancelled => "[CANCELLED]",
            _ => "[?]",
        };
        println!("  {symbol} {id}");
    }

    if report.all_succeeded() {
        println!("\nPipeline succeeded.");
    } else {
        println!("\nPipeline failed. Failed tasks: {:?}", report.failed_tasks());
    }
}
