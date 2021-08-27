use std::{
    ops::DerefMut,
    process::Child,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};

use killall::{kill, list_descendants};

use crate::errors::Error;

struct Shared {
    proc_info: Mutex<(Child, bool)>,
    cond_var: Condvar,
}
impl Shared {
    fn new(child: Child) -> Self {
        Self {
            proc_info: Mutex::new((child, false)),
            cond_var: Condvar::new(),
        }
    }
}

/// This function does its best to make sure the given child does not run longer
/// than the given timeout. To do so, it spawns a thread that periodically polls
/// the child for completion (sucessful or failed: I dont care [though I could]).
/// If the child process completes, the side thread notifies the main thread
/// via a cond var.
///
/// On the other hand, the main thread blocks on the condition variable until
/// either the timeout occurs or it gets notified by the conditional variable.
///
/// In case the timeout occurs, some cleanup is performed to make sure all
/// children processes are killed.
pub fn timeout(child: Child, timeout: Duration) -> Result<(), Error> {
    let shared = Arc::new(Shared::new(child));

    let shared2 = Arc::clone(&shared);
    thread::spawn(move || loop_until_process_is_finished(shared2));

    let shared = shared.as_ref();
    let lock = shared.proc_info.lock()?;
    let (mut guard, _) = shared
        .cond_var
        .wait_timeout_while(lock, timeout, |&mut (_, done)| !done)?;

    let (child, done) = guard.deref_mut();
    *done = true;
    maybe_cleanup(child)?;

    Ok(())
}

/// Loops until the process finishes and signals it through the conditional var
fn loop_until_process_is_finished(shared: Arc<Shared>) -> Result<(), Error> {
    loop {
        {
            let shared = shared.as_ref();
            let (ref mut child, ref mut done) = *shared.proc_info.lock()?;
            if *done {
                break;
            }
            if let Ok(Some(_status)) = child.try_wait() {
                *done = true;
                shared.cond_var.notify_all();
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
    Ok(())
}

/// Cleanup the potential zombie kids
fn maybe_cleanup(child: &mut Child) -> Result<(), Error> {
    if child.try_wait()?.is_none() {
        let childrens = list_descendants(child.id() as usize)?;
        for kid in childrens {
            kill(&kid)?;
        }
    }
    Ok(())
}
