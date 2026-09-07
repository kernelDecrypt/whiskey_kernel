use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::println;

pub type TaskFn = fn();

const MAX_TASKS: usize = 8;

#[derive(Copy, Clone)]
struct Task {
    name: &'static str,
    run: TaskFn,
    runs: usize,
    active: bool,
}

impl Task {
    const fn empty() -> Self {
        Self {
            name: "",
            run: empty_task,
            runs: 0,
            active: false,
        }
    }
}

struct TaskTable {
    inner: UnsafeCell<[Task; MAX_TASKS]>,
}

unsafe impl Sync for TaskTable {}

static TASKS: TaskTable = TaskTable {
    inner: UnsafeCell::new([Task::empty(); MAX_TASKS]),
};
static TASK_COUNT: AtomicUsize = AtomicUsize::new(0);
static CURRENT_INDEX: AtomicUsize = AtomicUsize::new(0);
static DEMO_TICK: AtomicUsize = AtomicUsize::new(0);

fn empty_task() {}

fn idle_task() {
    // Intentionally empty dont question the arts
}

fn demo_task() {
    let tick = DEMO_TICK.fetch_add(1, Ordering::Relaxed) + 1;
    if tick % 16 == 0 {
        println!("[task demo] tick {}", tick);
    }
}

pub fn init() {
    TASK_COUNT.store(0, Ordering::Relaxed);
    CURRENT_INDEX.store(0, Ordering::Relaxed);
    DEMO_TICK.store(0, Ordering::Relaxed);

    let tasks = unsafe { &mut *TASKS.inner.get() };
    for slot in tasks.iter_mut() {
        *slot = Task::empty();
    }

    let _ = register_task("idle", idle_task);
    let _ = register_task("demo", demo_task);
}

pub fn register_task(name: &'static str, run: TaskFn) -> bool {
    let count = TASK_COUNT.load(Ordering::Relaxed);
    if count >= MAX_TASKS {
        return false;
    }

    let tasks = unsafe { &mut *TASKS.inner.get() };
    let slot = &mut tasks[count];
    slot.name = name;
    slot.run = run;
    slot.runs = 0;
    slot.active = true;
    TASK_COUNT.store(count + 1, Ordering::Relaxed);
    true
}

pub fn run_scheduler_once() {
    let count = TASK_COUNT.load(Ordering::Relaxed);
    if count == 0 {
        return;
    }

    let index = CURRENT_INDEX.load(Ordering::Relaxed) % count;
    CURRENT_INDEX.store((index + 1) % count, Ordering::Relaxed);

    let tasks = unsafe { &mut *TASKS.inner.get() };
    let task = &mut tasks[index];
    if task.active {
        task.runs += 1;
        (task.run)();
    }
}

pub fn current_task_id() -> usize {
    CURRENT_INDEX.load(Ordering::Relaxed)
}

pub fn dump_status() {
    let count = TASK_COUNT.load(Ordering::Relaxed);
    println!("Registered tasks:");
    if count == 0 {
        println!("  (none)");
        return;
    }

    let tasks = unsafe { &*TASKS.inner.get() };
    for i in 0..count {
        let task = &tasks[i];
        if task.active {
            println!("  {} -> runs {}", task.name, task.runs);
        }
    }
}
