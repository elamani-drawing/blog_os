
use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc};
use core::task::Waker;
use crossbeam_queue::ArrayQueue;
use core::task::{Context, Poll};
use alloc::task::Wake;

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            //Pour créer un Executor, nous fournissons une fonction simple new. Nous choisissons une capacité de 100 pour le task_queue, ce qui devrait 
            //être plus que suffisant dans un avenir prévisible. Dans le cas où notre système aura plus de 100 tâches simultanées à un moment donné,
            // nous pouvons facilement augmenter cette taille.
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }
    ///Comme pour le SimpleExecutor, nous fournissons une spawnméthode sur notre Executortype qui ajoute une tâche donnée à la taskscarte et la réveille 
    /// immédiatement en poussant son ID vers le task_queue
    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        //S'il existe déjà une tâche avec le même ID dans la carte, la BTreeMap::insertméthode [ ] la renvoie. Cela ne devrait jamais arriver puisque chaque 
        //tâche a un identifiant unique, donc nous paniquons dans ce cas car cela indique un bogue dans notre code. 
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already in tasks");
        }
        //De même, nous paniquons lorsque le task_queueest plein car cela ne devrait jamais arriver si nous choisissons une taille de file d'attente suffisamment grande.
        self.task_queue.push(task_id).expect("queue pleine, augmenter la taille de task_queu");
    }

    //Pour exécuter toutes les tâches dans le task_queue, nous créons une run_ready_tasksméthode privée
    fn run_ready_tasks(&mut self) {
        // destructure `self` to avoid borrow checker errors
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;
        // bouclez sur toutes les tâches du task_queue, créez un réveil pour chaque tâche, puis interrogez-le
        while let Ok(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // task no longer exists
            };
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task done -> remove it and its cached waker
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    //Cette méthode appelle simplement la run_ready_tasksfonction dans une boucle. Bien que nous puissions théoriquement revenir de la fonction 
    //lorsque la taskscarte devient vide, cela ne se produirait jamais puisque notre keyboard_taskne se termine jamais, donc un simple loopdevrait suffire.
    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();  
        }
    }
    fn sleep_if_idle(&self) {
        //Pour éviter les conditions de concurrence, nous désactivons les interruptions avant de vérifier si le task_queueest vide.
        use x86_64::instructions::interrupts::{self, enable_and_hlt};

        interrupts::disable();
        // Si c'est le cas, nous utilisons la enable_and_hltfonction pour activer les interruptions et mettre le CPU en veille en une seule opération atomique. 
        //Dans le cas où la file d'attente n'est plus vide, cela signifie qu'une interruption a réveillé une tâche après son run_ready_tasksretour. Dans ce cas, 
        //nous activons à nouveau les interruptions et continuons directement l'exécution sans exécuter hlt.
        if self.task_queue.is_empty() {
            enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }
}