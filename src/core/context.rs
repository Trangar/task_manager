use crate::core::{AsyncCreateContext, Runtime, Scheduler, Task, ID};
use crate::sys::{AsyncNotifier, Events, SysRuntime};

pub struct Context<'a> {
    pub event: Events,
    runtime: &'a mut Runtime,
}

impl Context<'_> {
    pub(crate) fn new(runtime: &mut Runtime, event: Events) -> Context {
        Context { event, runtime }
    }

    pub fn register_async<'a>(
        &'a mut self,
        listener: &dyn AsyncNotifier,
    ) -> AsyncCreateContext<'a, Self> {
        AsyncCreateContext(self, listener.as_raw_fd())
    }
}

impl Scheduler for Context<'_> {
    fn register(&mut self, task: Box<dyn Task + 'static>) -> ID {
        self.runtime.register(task)
    }
    fn remove(&mut self, id: ID) {
        self.runtime.remove(id)
    }
    fn sys(&mut self) -> &mut SysRuntime {
        self.runtime.sys()
    }
}
