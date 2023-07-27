use self::scheduler::LazyTargetSchedler;
use crate::format::Target;
use ahash::AHashMap;
use std::{borrow::BorrowMut, sync::Arc};

pub mod scheduler;

pub struct Executer<'a> {
    pub thread_count: i32,
    pub all_targets: Arc<AHashMap<&'a str, &'a Target<'a>>>,
    targets_scheduler: Box<dyn scheduler::TargetScheduler<'a> + 'a>,
}

impl<'a> Executer<'a> {
    pub fn new(count: i32, targets: AHashMap<&'a str, &'a Target<'a>>) -> Executer<'a> {
        let arc = Arc::new(targets);
        Executer::<'a> {
            thread_count: count,
            targets_scheduler: Box::new(LazyTargetSchedler::<'a>::new(arc.clone())),
            all_targets: arc,
        }
    }

    pub fn execute(&'a mut self, targets: &Vec<String>) -> Result<(), ()> {
        for target in targets {
            (*self.targets_scheduler).borrow_mut().target(target);
        }

        return Ok(());
    }
}
