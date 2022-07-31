
///La Taskstructure est un wrapper de nouveau type autour d'un futur épinglé, alloué au tas et distribué dynamiquement avec le type vide ()en sortie.

use core::{future::Future, pin::Pin};
use alloc::boxed::Box;
use core::task::{Context, Poll};

pub mod simple_executor;
pub struct Task {
    ///Nous exigeons que le futur associé à une tâche renvoie (). Cela signifie que les tâches ne renvoient aucun résultat, elles sont juste 
    ///exécutées pour ses effets secondaires.

    ///Le dynmot-clé indique que nous stockons un objet trait dans le fichier Box. Cela signifie que les méthodes sur le futur sont distribuées dynamiquement , 
    ///ce qui permet de stocker différents types de futurs dans le Tasktype. Ceci est important car chacun async fna son propre type et nous voulons pouvoir 
    ///créer plusieurs tâches différentes.

    ///Comme nous l'avons appris dans la section sur l'épinglage , le Pin<Box>type garantit qu'une valeur ne peut pas être déplacée en mémoire en la plaçant 
    ///sur le tas et en empêchant la création de &mutréférences à celle-ci. Ceci est important car les futurs générés par async/wait peuvent être 
    ///auto-référentiels, c'est-à-dire contenir des pointeurs sur eux-mêmes qui seraient invalidés lorsque le futur est déplacé.
    future: Pin<Box<dyn Future<Output = ()>>>,
}


impl Task {
    ///Pour permettre la création de nouvelles Taskstructures à partir de futures, nous créons une newfonction :
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        //La fonction prend un futur arbitraire avec le type de sortie ()et l'épingle en mémoire via la Box::pinfonction. Ensuite, il enveloppe le futur 
        //encadré dans la Taskstructure et le renvoie. La 'staticdurée de vie est requise ici car le retour Taskpeut vivre pendant une durée arbitraire, 
        //de sorte que le futur doit également être valide pour cette durée.
        Task {
            future: Box::pin(future),
        }
    }
    ///Nous ajoutons également une pollméthode pour permettre à l'exécuteur d'interroger le futur stocké
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        //Puisque la pollméthode du Futuretrait s'attend à être appelée sur un Pin<&mut T>type, nous utilisons d'abord la Pin::as_mutméthode pour 
        //convertir le self.futurechamp de type Pin<Box<T>>. Ensuite, nous appelons pollle champ converti self.futureet renvoyons le résultat. 
        //Étant donné que la Task::pollméthode ne doit être appelée que par l'exécuteur que nous créons dans un instant, nous gardons la fonction 
        //privée pour le taskmodule.
        self.future.as_mut().poll(context)
    }
}