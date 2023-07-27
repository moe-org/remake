
use crate::format::Target;
use spin;
use spin::MutexGuard;
use ahash::AHashMap;
use ahash::AHashSet;

pub trait TargetScheduler<'a>{
    // if return None, it means all the targets have been taken
    fn get_next_target(&'a mut self) -> Option<&'a Target>;
    // if all the targets have been taken,return true,
    // and the get_next_target return None.
    //
    // but it doesn't mean that all the tasks have been done.
    // Some threads may still work.
    //
    fn is_done(&'a self) -> bool;
    // call this when a thread done a target
    fn done_target(&'a mut self,target:&'a str);

    fn target(&'a mut self,target:&'a Target<'a>);
}

pub struct LazyTargetSchedler<'a>{
    all_targets:spin::Mutex<AHashMap<&'a str,&'a Target<'a>>>,
    todo_targets:spin::Mutex<Vec<&'a Target<'a>>>,
    unresolved_targets:spin::Mutex<Vec<&'a Target<'a>>>,
    done_targets : spin::Mutex<AHashSet<&'a str>>,
    done:bool
}

impl<'a> LazyTargetSchedler<'a>{
    fn resolve_target(&self,target:&'a Target<'a>,
                          todo_targets:&mut MutexGuard<Vec<&'a Target<'a>>>,
                          unresolved_targets:&mut MutexGuard<Vec<&'a Target<'a>>>){
        // check
        let deps = target.dependences; 
        todo_targets.push(target);
        for item in deps{
            unresolved_targets.push(
                self.all_targets.lock().get(item).unwrap()
                );
        }
    }
    fn get_runable_target(&self,todo_targets:&MutexGuard<Vec<&'a Target<'a>>>,
                              done_targets:&MutexGuard<AHashSet<&'a str>>) -> Option<&'a Target<'a>>{
        for todo in todo_targets.iter(){
            let mut is_done = true;
            for dependence in todo.dependences.iter(){
               if !(*done_targets).contains(dependence) {
                   is_done  = false;
                   break;
               }
            }
            if is_done{ 
                return Some(todo);
            }
        }
        return None;
    }
    pub fn new(all_targets:AHashMap<&'a str,&'a Target<'a>>)->LazyTargetSchedler<'a>{
        LazyTargetSchedler { 
            all_targets:spin::Mutex::new(all_targets),
            todo_targets: spin::Mutex::new(Vec::new()), 
            unresolved_targets: spin::Mutex::new(Vec::new()), 
            done_targets: spin::Mutex::new(AHashSet::new()), 
            done: false 
        }
    }
}

impl<'a> TargetScheduler<'a> for LazyTargetSchedler<'a>{
    fn get_next_target(&'a mut self) -> Option<&'a Target>{
        let mut todos = self.todo_targets.lock();
        let dones = self.done_targets.lock();
        let mut got = self.get_runable_target(&todos,&dones);
        while got.is_none(){
            let mut un = self.unresolved_targets.lock();
            if un.is_empty(){
                self.done = true;
                return None;
            }
            else{
                let got = un.pop();
                self.resolve_target(got.unwrap(),
                    &mut todos,
                    &mut un
                );
            }
            got = self.get_runable_target(&todos,&dones);
        }
        return Some(got.unwrap());
    }
    fn done_target(&mut self,target:&'a str){
        let mut dones = self.done_targets.lock();
        dones.insert(target);
    }
    fn is_done(&self) -> bool{
        return self.done;
    }
    fn target(&'a mut self,target:&'a Target<'a>){
        self.unresolved_targets.lock().push(target)
    }
}
