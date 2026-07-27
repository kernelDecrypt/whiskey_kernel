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

static mut TASKS: [Task; MAX_TASKS] = [Task::empty(); MAX_TASKS];
static mut TASK_COUNT: usize = 0;
static mut CURRENT_INDEX: usize = 0;
static mut DEMO_TICK: usize = 0;

fn empty_task() {}

fn idle_task() {
    // Intentionally empty dont question the arts
}

fn demo_task() {
    unsafe {
        DEMO_TICK += 1;
        if DEMO_TICK % 16 == 0 {
            println!("[task demo] tick {}", DEMO_TICK);
        }
    }
}

pub fn init() {
    unsafe {
        TASK_COUNT = 0;
        CURRENT_INDEX = 0;
        for slot in TASKS.iter_mut() {
            *slot = Task::empty();
        }
    }

    let _ = register_task("idle", idle_task);
    let _ = register_task("demo", demo_task);
}

pub fn register_task(name: &'static str, run: TaskFn) -> bool {
    unsafe {
        if TASK_COUNT >= MAX_TASKS {
            return false;
        }

        let slot = &mut TASKS[TASK_COUNT];
        slot.name = name;
        slot.run = run;
        slot.runs = 0;
        slot.active = true;
        TASK_COUNT += 1;
        true
    }
}

pub fn run_scheduler_once() {
    unsafe {
        if TASK_COUNT == 0 {
            return;
        }

        let index = CURRENT_INDEX % TASK_COUNT;
        CURRENT_INDEX = (index + 1) % TASK_COUNT;

        let task = &mut TASKS[index];
        if task.active {
            task.runs += 1;
            (task.run)();
        }
    }
}

pub fn current_task_id() -> usize {
    unsafe { CURRENT_INDEX }
}

pub fn dump_status() {
    unsafe {
        println!("Registered tasks:");
        if TASK_COUNT == 0 {
            println!("  (none)");
            return;
        }

        for i in 0..TASK_COUNT {
            let task = &TASKS[i];
            if task.active {
                println!("  {} -> runs {}", task.name, task.runs);
            }
        }
    }
}
