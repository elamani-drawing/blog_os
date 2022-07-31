use super::Task;
use alloc::collections::VecDeque;
use core::task::{Waker, RawWaker};
use core::task::RawWakerVTable;
use core::task::{Context, Poll};

///La structure contient un seul task_queuechamp de type VecDeque, qui est essentiellement un vecteur permettant de pousser et de faire apparaître des opérations 
/// aux deux extrémités. L'idée derrière l'utilisation de ce type est que nous insérons de nouvelles tâches via la spawnméthode à la fin et que nous déployons 
/// la tâche suivante pour une exécution à partir de l'avant. De cette façon, nous obtenons une simple file d'attente FIFO ( "first in, first out" ).

pub struct SimpleExecutor {
    task_queue: VecDeque<Task>,
}

impl SimpleExecutor {
    pub fn new() -> SimpleExecutor {
        SimpleExecutor {
            task_queue: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task)
    }
    //La fonction utilise une while letboucle pour gérer toutes les tâches du fichier task_queue. Pour chaque tâche, il crée d'abord un Contexttype en encapsulant
    // une Wakerinstance renvoyée par notre dummy_wakerfonction. Ensuite, il appelle la Task::pollméthode avec this context. Si la pollméthode renvoie Poll::Ready,
    // la tâche est terminée et nous pouvons continuer avec la tâche suivante. Si la tâche est toujours Poll::Pending, nous l'ajoutons à nouveau à l'arrière de 
    //la file d'attente afin qu'elle soit à nouveau interrogée lors d'une itération de boucle ultérieure.
    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {} // task done
                Poll::Pending => self.task_queue.push_back(task),
            }
        }
    }
}


fn dummy_raw_waker() -> RawWaker {
    //Tout d'abord, nous définissons deux fonctions internes nommées no_opet clone

    //La no_opfonction prend un *const ()pointeur et ne fait rien. 
    fn no_op(_: *const ()) {}
    //La clonefonction prend également un *const ()pointeur et renvoie un nouveau RawWakeren appelant dummy_raw_wakerà nouveau.
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}

fn dummy_waker() -> Waker {
    //La from_rawfonction n'est pas sûre car un comportement indéfini peut se produire si le programmeur ne respecte pas les exigences documentées de RawWaker. 
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}