use crate::core::{Context, Task, ID};
use crate::sys::{AsyncNotifier, SysRuntime};

pub struct Runtime {
    sys: SysRuntime,
    tasks: Vec<Option<(ID, Box<dyn Task>)>>,
    task_ids: Vec<ID>,
}

impl Runtime {
    pub fn create() -> std::io::Result<Runtime> {
        let sys = SysRuntime::create()?;
        Ok(Runtime {
            sys,
            tasks: Vec::new(),
            task_ids: Vec::new(),
        })
    }

    pub fn register_async<'a>(
        &'a mut self,
        listener: &dyn AsyncNotifier,
    ) -> AsyncCreateContext<'a, Self> {
        AsyncCreateContext(self, listener.as_raw_fd())
    }

    pub fn run(mut self) -> std::io::Result<()> {
        while self.tasks.iter().any(|t| t.is_some()) {
            let events = self.sys.wait(None)?;
            for (event, id) in events {
                if let Some((task_id, task)) = self.tasks.get_mut(id.index()).and_then(|o| o.take())
                {
                    if id.is_same_generation(task_id) {
                        let mut context = Context::new(&mut self, event);
                        task.execute(&mut context);
                    }
                }
            }
        }
        eprintln!("Runtime ran out of tasks!");
        Ok(())
    }
}

pub trait Scheduler {
    fn register(&mut self, task: Box<dyn Task + 'static>) -> ID;
    fn remove(&mut self, id: ID);
    fn sys(&mut self) -> &mut SysRuntime;
}

impl Scheduler for Runtime {
    fn register(&mut self, task: Box<dyn Task + 'static>) -> ID {
        let id = if let Some(id) = self.task_ids.pop() {
            let id = id.next_generation();
            let index = id.index();

            debug_assert!(self.tasks.len() > index);
            debug_assert!(self.tasks.get(index).unwrap().is_none());
            id
        } else {
            let id = ID::new(self.tasks.len() as u32, 0);
            self.tasks.push(None);
            id
        };

        if let Some(old_task) = self.tasks.get_mut(id.index()) {
            *old_task = Some((id, task));
        } else {
            unreachable!();
        }
        id
    }

    fn remove(&mut self, id: ID) {
        let index = id.index();
        debug_assert!(self.tasks.len() > index);
        debug_assert!(self.tasks.get(index).unwrap().is_none());
        if let Some(old_task) = self.tasks.get_mut(id.index()) {
            *old_task = None;
        }
        self.task_ids.push(id);
    }

    fn sys(&mut self) -> &mut SysRuntime {
        &mut self.sys
    }
}

#[must_use]
pub struct AsyncCreateContext<'a, T>(pub(crate) &'a mut T, pub(crate) std::os::unix::io::RawFd);

impl<T: Scheduler> AsyncCreateContext<'_, T> {
    pub fn with(self, task: impl crate::core::Task + 'static) -> std::io::Result<()> {
        self.with_boxed(Box::new(task))
    }

    pub fn with_boxed(self, task: Box<dyn Task>) -> std::io::Result<()> {
        let id = self.0.register(task);
        match self.0.sys().register(id, self.1) {
            Ok(()) => Ok(()),
            Err(e) => {
                eprintln!("{:?}", e);
                self.0.remove(id);
                Err(e)
            }
        }
    }
}
