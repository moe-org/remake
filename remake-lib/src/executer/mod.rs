use crate::format::Target;
use std::sync::Arc;

use self::scheduler::LazyTargetSchedler;
use ahash::AHashMap;
pub mod scheduler;


pub struct Executer<'a>{
    pub thread_count:i32,
    pub all_targets:AHashMap<&'a str,&'a Target<'a>>,
    targets_scheduler:Arc<dyn scheduler::TargetScheduler<'a>>
}

impl<'a> Executer<'a>{
    pub fn new(count:i32,all_targets:AHashMap<&'a str,&'a Target<'a>>) -> Executer<'a>{
        return Executer{
            thread_count:count,
            all_targets:all_targets.clone(),
            targets_scheduler:Arc::new(LazyTargetSchedler::new(all_targets))
        }
    }

    pub fn execute(&'a mut self,targets:&'a [&'a Target<'a>]) -> Result<(),()>{
        for target in targets{
            Arc::get_mut(&mut self.targets_scheduler).unwrap().target(target);
        }
        

        return Ok(());
    }
}

