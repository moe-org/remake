use crate::format::Target;
use ahash::AHashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use self::scheduler::TargetScheduler;

pub mod scheduler;

/// This is the executer of the targets
pub struct Executer {
    pub thread_count: u32,
    pub all_targets: Arc<AHashMap<Arc<String>, Arc<Target>>>,
    /// If you want to print some useful message to user,set this.
    pub logger: Arc<dyn Fn(&str) -> () + Sync + Send>,
    targets_scheduler: scheduler::TargetScheduler,
}

impl Executer {
    /// Create a new ececuter from targets.
    pub fn new(count: u32, targets: AHashMap<Arc<String>, Arc<Target>>) -> Executer {
        let arc = Arc::new(targets);
        Executer {
            thread_count: count,
            targets_scheduler: TargetScheduler::new(arc.clone()),
            logger: Arc::new(|_| -> () { () }),
            all_targets: arc,
        }
    }

    /// Parse the dependences of the targets and execute them at a sequence.
    pub fn execute(&mut self, targets: &Vec<String>) -> spin::Mutex<Vec<String>> {
        // resolve targets
        for target in targets {
            self.targets_scheduler.target(target);
        }
        let errors: spin::Mutex<Vec<String>> =
            spin::Mutex::new(Vec::with_capacity(self.thread_count as usize));
        let dur = Duration::from_millis(10);

        // begin to work
        thread::scope(|s| {
            let mut threads: Vec<thread::ScopedJoinHandle<_>> = Vec::new();
            let scheduler = &self.targets_scheduler;

            for _ in 0..self.thread_count {
                let j = s.spawn(|| {
                    let id = thread::current().id();
                    loop {
                        let target = scheduler.get_next_target();

                        match target {
                            None => {
                                return {
                                    (*self.logger)(format!("Thread {} Exit", id.as_u64()).as_str());
                                }
                            }
                            Some(target) => {
                                for command in target.commands.iter() {
                                    let run = command.run();

                                    if run.is_err() {
                                        scheduler.report_error();
                                        errors.lock().push(run.err().unwrap().to_string());
                                    }
                                }
                                scheduler.done_target(target.name.clone());

                                (*self.logger)(
                                    format!("Thread {} Executed {}", id.as_u64(), target.name)
                                        .as_str(),
                                );
                            }
                        }
                    }
                });

                threads.push(j);
            }

            loop {
                let mut all_finished = true;

                for t in &threads {
                    if !t.is_finished() {
                        all_finished = false;
                        break;
                    }
                }

                if all_finished {
                    return;
                }

                thread::sleep(dur);
            }
        });

        // return
        return errors;
    }
}
