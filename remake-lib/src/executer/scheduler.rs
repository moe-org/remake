use crate::format::Target;
use ahash::AHashMap;
use ahash::AHashSet;
use spin;
use spin::MutexGuard;
use std::collections::VecDeque;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// The is a schedler
pub struct TargetScheduler {
    all_targets: Arc<AHashMap<Arc<String>, Arc<Target>>>,
    todo_targets: spin::Mutex<VecDeque<Arc<Target>>>,
    unresolved_targets: spin::Mutex<Vec<Arc<Target>>>,
    done_targets: spin::Mutex<AHashSet<Arc<String>>>,
    done: AtomicBool,
    error: AtomicBool,
}

impl TargetScheduler {
    /// Resolve a target.
    /// This will deal with wheir dependences.
    fn resolve_target(
        &self,
        target: Arc<Target>,
        todo_targets: &mut MutexGuard<VecDeque<Arc<Target>>>,
        unresolved_targets: &mut MutexGuard<Vec<Arc<Target>>>,
    ) {
        // check
        let deps = &target.dependences;
        todo_targets.push_back(target.clone());
        for item in deps.iter() {
            unresolved_targets.push(self.all_targets.get(item).unwrap().clone());
        }
    }

    /// Get a target that has no dependence or all its dependences have executed.
    fn get_runable_target(
        &self,
        todo_targets: &MutexGuard<VecDeque<Arc<Target>>>,
        done_targets: &MutexGuard<AHashSet<Arc<String>>>,
    ) -> Option<Arc<Target>> {
        for todo in todo_targets.iter().rev() {
            let mut is_done = true;
            for dependence in todo.dependences.iter() {
                if !(*done_targets).contains(dependence) {
                    is_done = false;
                    break;
                }
            }
            if is_done {
                return Some(todo.clone());
            }
        }
        return None;
    }

    /// Create a new scheduler from targets
    pub fn new(all_targets: Arc<AHashMap<Arc<String>, Arc<Target>>>) -> TargetScheduler {
        TargetScheduler {
            all_targets,
            todo_targets: spin::Mutex::new(VecDeque::with_capacity(32)),
            unresolved_targets: spin::Mutex::new(Vec::with_capacity(32)),
            done_targets: spin::Mutex::new(AHashSet::with_capacity(32)),
            done: AtomicBool::new(false),
            error: AtomicBool::new(false),
        }
    }

    /// Get the next target that will be executed.
    pub fn get_next_target(&self) -> Option<Arc<Target>> {
        if self.is_done() {
            return None;
        }

        let mut todos = self.todo_targets.lock();
        let dones = self.done_targets.lock();
        let mut got = self.get_runable_target(&todos, &dones);

        while got.is_none() {
            let mut un = self.unresolved_targets.lock();
            if un.is_empty() {
                self.done.store(true, Ordering::Relaxed);
                return None;
            } else {
                let got = un.pop();
                self.resolve_target(got.unwrap(), &mut todos, &mut un);
            }
            got = self.get_runable_target(&todos, &dones);
        }

        return Some(got.unwrap());
    }

    pub fn report_error(&self) {
        self.error.store(true, Ordering::SeqCst);
    }

    /// Mark a target was executed.
    pub fn done_target(&self, target: Arc<String>) {
        let mut dones = self.done_targets.lock();
        dones.insert(target);
    }

    /// Detect if all the targets were get. Or there is a break.
    pub fn is_done(&self) -> bool {
        return self.done.load(Ordering::SeqCst) || self.error.load(Ordering::SeqCst);
    }

    /// Mark a target that you want to execute.
    pub fn target(&self, target: &String) {
        let t = self.all_targets.get(target);
        self.unresolved_targets.lock().push(t.unwrap().clone());
    }
}
