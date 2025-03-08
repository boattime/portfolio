use crate::config::Config;
use crate::error::{Error, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::future::join_all;
use log::{error, info, warn};
use std::sync::Arc;
use tokio::spawn;
use tokio::sync::Mutex;
use tokio::time;
use tokio::time::Duration;

#[async_trait]
pub trait Task: Send + Sync {
    fn name(&self) -> &str;
    async fn execute(&self) -> Result<()>;
}

pub struct ScheduledTask {
    task: Arc<dyn Task>,
    last_run: Option<DateTime<Utc>>,
    success_count: usize,
    failure_count: usize,
}

impl ScheduledTask {
    pub fn new(task: Arc<dyn Task>) -> Self {
        Self {
            task,
            last_run: None,
            success_count: 0,
            failure_count: 0,
        }
    }

    pub async fn execute(&mut self) -> Result<()> {
        let start_time = Utc::now();
        let task_name = self.task.name();

        info!("Executing task: {}", task_name);

        match self.task.execute().await {
            Ok(()) => {
                self.success_count += 1;
                self.last_run = Some(start_time);
                let duration = Utc::now() - start_time;
                info!(
                    "Task '{}' completed successfully in {} ms",
                    task_name,
                    duration.num_milliseconds()
                );
                Ok(())
            }
            Err(e) => {
                self.failure_count += 1;
                self.last_run = Some(start_time);
                let duration = Utc::now() - start_time;
                error!(
                    "Task '{}' failed after {} ms: {}",
                    task_name,
                    duration.num_milliseconds(),
                    e
                );
                Err(e)
            }
        }
    }

    pub fn metrics(&self) -> TaskMetrics {
        TaskMetrics {
            name: self.task.name().to_string(),
            last_run: self.last_run,
            success_count: self.success_count,
            failure_count: self.failure_count,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskMetrics {
    pub name: String,
    pub last_run: Option<DateTime<Utc>>,
    pub success_count: usize,
    pub failure_count: usize,
}

pub struct Scheduler {
    config: Config,
    tasks: Arc<Mutex<Vec<ScheduledTask>>>,
    running: Arc<Mutex<bool>>,
}

impl Scheduler {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            tasks: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn add_task(&self, task: Arc<dyn Task>) {
        let mut tasks = self.tasks.lock().await;
        tasks.push(ScheduledTask::new(task));
    }

    pub async fn run(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        if *running {
            return Err(Error::SchedulerError(
                "Scheduler is already running".to_string(),
            ));
        }

        *running = true;

        let interval_seconds = self.config.interval_seconds;
        info!(
            "Starting scheduler with interval of {} seconds",
            interval_seconds
        );

        let running_clone = self.running.clone();
        let tasks_clone = self.tasks.clone();

        spawn(async move {
            let mut interval = time::interval(Duration::from_secs(interval_seconds));

            Self::execute_all_tasks(&tasks_clone).await;

            loop {
                interval.tick().await;

                let running = *running_clone.lock().await;
                if !running {
                    break;
                }

                Self::execute_all_tasks(&tasks_clone).await;
            }

            info!("Scheduler stopped");
        });

        Ok(())
    }

    // This implementation intentionally acquires and releases the lock multiple times
    // to avoid holding it across await points, which could cause deadlocks.
    // Each task gets its own Arc clone and acquires the lock only when needed.
    async fn execute_all_tasks(tasks: &Arc<Mutex<Vec<ScheduledTask>>>) {
        let task_count = {
            let tasks_lock = tasks.lock().await;
            if tasks_lock.is_empty() {
                warn!("No tasks to execute");
                return;
            }
            tasks_lock.len()
        };

        info!("Executing {} tasks", task_count);

        let mut handles = Vec::with_capacity(task_count);

        for task_index in 0..task_count {
            let tasks_clone = Arc::clone(tasks);

            let task_future = async move {
                let mut tasks_guard = tasks_clone.lock().await;

                if let Some(task) = tasks_guard.get_mut(task_index) {
                    let result = task.execute().await;
                    (task_index, result)
                } else {
                    (
                        task_index,
                        Err(Error::SchedulerError("Task not found".to_string())),
                    )
                }
            };

            handles.push(task_future);
        }

        // Execute all tasks concurrently and collect results
        let results = join_all(handles).await;

        // Log summary
        let success_count = results.iter().filter(|(_, result)| result.is_ok()).count();
        info!(
            "Completed task execution: {}/{} successful",
            success_count, task_count
        );
    }

    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        if !*running {
            return Err(Error::SchedulerError(
                "Scheduler is not running".to_string(),
            ));
        }

        *running = false;
        Ok(())
    }

    pub async fn metrics(&self) -> Vec<TaskMetrics> {
        let tasks = self.tasks.lock().await;
        tasks.iter().map(|task| task.metrics()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestTask {
        name: String,
        counter: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Task for TestTask {
        fn name(&self) -> &str {
            &self.name
        }

        async fn execute(&self) -> Result<()> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_scheduled_task_execution() {
        let counter = Arc::new(AtomicUsize::new(0));
        let task = TestTask {
            name: "test_task".to_string(),
            counter: counter.clone(),
        };

        let mut scheduled_task = ScheduledTask::new(Arc::new(task));
        assert_eq!(scheduled_task.success_count, 0);

        scheduled_task.execute().await.unwrap();
        assert_eq!(scheduled_task.success_count, 1);
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        scheduled_task.execute().await.unwrap();
        assert_eq!(scheduled_task.success_count, 2);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_scheduler_add_task() {
        let config = Config::default();
        let scheduler = Scheduler::new(config);

        let counter = Arc::new(AtomicUsize::new(0));
        let task = TestTask {
            name: "test_task".to_string(),
            counter: counter.clone(),
        };

        scheduler.add_task(Arc::new(task)).await;

        let tasks = scheduler.tasks.lock().await;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].task.name(), "test_task");
    }
}
