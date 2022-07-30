//cf voir allocateur de bosses/bump
use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;
//Le nextpointeur ne se déplace que dans une seule direction et ne distribue donc jamais deux fois la même région de mémoire. 
//Lorsqu'il atteint la fin du tas, plus aucune mémoire ne peut être allouée, ce qui entraîne une erreur de mémoire insuffisante lors de la prochaine allocation.
pub struct BumpAllocator {
    //Les champs heap_startet heap_endgardent une trace de la limite inférieure et supérieure de la région de mémoire de tas. L'appelant doit s'assurer 
    //que ces adresses sont valides, sinon l'allocateur renverrait une mémoire invalide. Pour cette raison, la initfonction doit être unsafeà appeler.
    heap_start: usize,
    heap_end: usize,
    next: usize,//Le but du nextchamp est de toujours pointer sur le premier octet inutilisé du tas, c'est-à-dire l'adresse de début de la prochaine allocation.
    allocations: usize,
    //Le allocationschamp est un simple compteur des allocations actives dans le but de réinitialiser l'allocateur après la libération de la dernière allocation. Il est initialisé à 0.
}

impl BumpAllocator {
    /// Creates a new empty bump allocator.
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    //Nous avons choisi de créer une initfonction séparée au lieu d'effectuer l'initialisation directement dans newafin de garder l'interface identique à l'allocateur fourni par le linked_list_allocatorcrate. 
    //De cette façon, les répartiteurs peuvent être commutés sans modifications de code supplémentaires.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

//Comme expliqué dans le post précédent , tous les allocations de tas doivent implémenter le GlobalAlloctrait
unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    //La première étape pour les deux alloc et dealloc consiste à appeler la Mutex::lockméthode via le innerchamp pour obtenir une référence mutable au type d'allocateur enveloppé.
    //L'instance reste verrouillée jusqu'à la fin de la méthode, de sorte qu'aucune course aux données ne puisse se produire dans des contextes multithreads (nous ajouterons bientôt le support des threads).
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock(); // get a mutable reference

        let alloc_start = align_up(bump.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > bump.heap_end {
            ptr::null_mut() // out of memory
        } else {
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock(); // get a mutable reference

        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}