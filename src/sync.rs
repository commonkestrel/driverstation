use std::{future::Future, io};

use tokio::task::JoinHandle;

#[derive(Debug)]
pub enum Runtime {
    Runtime(tokio::runtime::Runtime),
    Handle(tokio::runtime::Handle),
}

impl Clone for Runtime {
    fn clone(&self) -> Self {
        match self {
            Runtime::Runtime(rt) => Runtime::Handle(rt.handle().clone()),
            Runtime::Handle(handle) => Runtime::Handle(handle.clone()),
        }
    }
}

impl Runtime {
    pub fn current() -> io::Result<Runtime> {
        match tokio::runtime::Handle::try_current() {
            Ok(rt) => Ok(Runtime::Handle(rt)),
            Err(_) => Ok(Runtime::Runtime(tokio::runtime::Runtime::new()?)),
        }
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        match self {
            Runtime::Runtime(rt) => rt.block_on(future),
            Runtime::Handle(rt) => rt.block_on(future),
        }
    }

    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        match self {
            Runtime::Handle(rt) => rt.spawn(future),
            Runtime::Runtime(rt) => rt.spawn(future),
        }
    }
}
