pub mod definition;

use crate::errors::RuntimeError;
use ahash::AHashMap;
use std::{ffi::OsStr, sync::atomic::AtomicBool, sync::Arc, sync::RwLock};

/// This stand for a target.
/// A target contains its name,dependences and commands.
/// The dependences is also a name refer to another target.
pub struct Target {
    pub name: Arc<String>,
    pub dependences: Arc<Vec<String>>,
    pub commands: Arc<Vec<CommandsRunable>>,
}

/// This stand for a command that will be executed.
pub struct Command {
    /// The executable file path
    pub executable: Arc<String>,
    /// The arguments of the command
    pub arguments: Vec<String>,
    /// If true,the run() will always return Ok
    pub ignore_error: AtomicBool,
    /// The environment variables
    pub environments: RwLock<AHashMap<String, String>>,
    /// The work directory of the command
    pub work_dir: Arc<String>,
}

/// The commands Runable contains a set of commands.
pub struct CommandsRunable {
    pub command: Arc<Command>,
}

impl Command {
    /// Execute a command
    pub fn run(&self) -> Result<(), RuntimeError> {
        // set up command
        let mut command = std::process::Command::new(self.executable.as_str());

        command.args(self.arguments.iter());

        command.current_dir(self.work_dir.as_str());

        let envs = self.environments.read().unwrap();

        for env in envs.iter() {
            command.env(OsStr::new(env.0.as_str()), OsStr::new(env.1.as_str()));
        }

        // run
        let child = command.spawn();

        match child {
            Ok(mut ret) => {
                let exit_status = ret.wait();

                if self.ignore_error.load(std::sync::atomic::Ordering::SeqCst) {
                    return Ok(());
                } else {
                    match exit_status {
                        Ok(status) => {
                            if status.success() {
                                return Ok(());
                            } else {
                                return Err(RuntimeError {
                                    source: None,
                                    command: Some(format!(
                                        "{} {:#?}",
                                        self.executable, self.arguments
                                    )),
                                    reason: Some(format!(
                                        "the program executed but return {}",
                                        status.code().unwrap()
                                    )),
                                });
                            }
                        }
                        Err(err) => {
                            return Err(RuntimeError {
                                source: Some(Arc::new(err)),
                                command: Some(format!("{} {:#?}", self.executable, self.arguments)),
                                reason: Some(String::from("can not execute the program")),
                            });
                        }
                    }
                }
            }
            Err(err) => {
                return Err(RuntimeError {
                    source: Some(Arc::new(err)),
                    command: Some(format!("{} {:#?}", self.executable, self.arguments)),
                    reason: Some(String::from("can not start the program")),
                })
            }
        }
    }
}

impl CommandsRunable {
    /// Execute all the commands
    pub fn run(&self) -> Result<(), RuntimeError> {
        let result = self.command.run();

        if result.is_err() {
            return result;
        }

        return Ok(());
    }
}
