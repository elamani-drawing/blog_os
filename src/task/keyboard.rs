use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::{pin::Pin, task::{Poll, Context}};
use futures_util::stream::Stream;
use futures_util::task::AtomicWaker;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();


use crate::println;

/// Called by the keyboard interrupt handler
///
/// Must not block or allocate.
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            //Le seul changement que nous avons effectué est d'ajouter un appel à WAKER.wake()si le push vers la file d'attente de scancode réussit. 
            //Si un waker est enregistré dans le WAKERstatique, cette méthode appellera la wakeméthode portant le même nom, qui notifiera l'exécuteur.
            // Sinon, l'opération est un no-op, c'est-à-dire que rien ne se passe.
            WAKER.wake(); 
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

pub struct ScancodeStream {
    //Le but du _privatechamp est d'empêcher la construction de la structure depuis l'extérieur du module. Cela fait de la newfonction le seul moyen 
    //de construire le type
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}


impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        //Nous utilisons d'abord la OnceCell::try_getméthode pour obtenir une référence à la file d'attente de scancode initialisée. Cela ne devrait jamais 
        //échouer puisque nous initialisons la file d'attente dans la newfonction, nous pouvons donc utiliser la expectméthode en toute sécurité pour paniquer 
        //si elle n'est pas initialisée.
        let queue = SCANCODE_QUEUE.try_get().expect("scancode queue not initialized");

        //Si le premier appel à queue.pop()échoue, la file d'attente est potentiellement vide. Seulement potentiellement parce que le gestionnaire d'interruptions
        // a peut-être rempli la file d'attente de manière asynchrone immédiatement après la vérification. Étant donné que cette condition de concurrence peut
        // se reproduire lors de la prochaine vérification, nous devons enregistrer le Wakerdans le WAKERstatique avant la deuxième vérification. De cette 
        //façon, un réveil peut se produire avant notre retour Poll::Pending, mais il est garanti que nous obtenons un réveil pour tous les scancodes poussés
        // après la vérification.
        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());

        // Ensuite, nous utilisons la ArrayQueue::pop méthode pour essayer d'obtenir l'élément suivant de la file d'attente. Si cela réussit, nous renvoyons 
        //le scancode enveloppé dans Poll::Ready(Some(…)). S'il échoue, cela signifie que la file d'attente est vide. Dans ce cas, nous retournons Poll::Pending.
        match queue.pop() {
            Ok(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}
