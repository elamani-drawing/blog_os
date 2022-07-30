
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};
/*
use linked_list_allocator::LockedHeap;
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
*/
//pour utiliser lalocator bump au lieu de linked_list 
//point fort rapide mais inconvenient : une seule allocation de longue durée suffit pour empecher la éutilisation de la memoir
use bump::BumpAllocator;

#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());


pub struct Dummy;
pub mod bump;

//La fonction prend des références mutables à a Mapperet à une FrameAllocatorinstance, toutes deux limitées à des pages de 4 Ko en utilisant Size4KiBcomme paramètre générique. La valeur de retour de la fonction est a Resultavec le type d'unité ()comme variante de succès et a MapToErrorcomme variante d'erreur, qui est le type d'erreur renvoyé par la Mapper::map_tométhode. La réutilisation du type d'erreur a ici du sens car la map_tométhode est la principale source d'erreurs dans cette fonction.
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    //Nous allouons un cadre physique auquel la page doit être mappée à l'aide de la FrameAllocator::allocate_frameméthode. Cette méthode revient Nonelorsqu'il ne reste plus d'images. Nous traitons ce cas en le mappant à une MapToError::FrameAllocationFailederreur via la Option::ok_orméthode, puis appliquons l' opérateur de point d'interrogation pour revenir tôt en cas d'erreur.
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
            //Nous définissons le PRESENTdrapeau requis et le WRITABLEdrapeau de la page. Avec ces drapeaux, les accès en lecture et en écriture sont autorisés, ce qui est logique pour la mémoire de tas.
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            //Nous utilisons la Mapper::map_tométhode de création du mappage dans la table des pages actives. La méthode peut échouer, nous utilisons donc à nouveau l' opérateur de point d'interrogation pour transmettre l'erreur à l'appelant. En cas de succès, la méthode renvoie une MapperFlushinstance que nous pouvons utiliser pour mettre à jour le tampon de recherche de traduction à l'aide de la flushméthode.
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        };
    }
    //il renvoie toujours une erreur sur alloc. Pour résoudre ce problème, nous devons initialiser l'allocateur après avoir créé le tas
    //Nous utilisons la lockméthode sur le spinlock interne du LockedHeaptype pour obtenir une référence exclusive à l' Heapinstance enveloppée, sur laquelle nous appelons ensuite la initméthode avec les limites du tas comme arguments. Il est important que nous initialisions le tas après avoir mappé les pages du tas, car la initfonction essaie déjà d'écrire dans la mémoire du tas.
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

//La première étape consiste à définir une région de mémoire virtuelle pour le tas. Nous pouvons choisir n'importe quelle plage d'adresses virtuelles que nous aimons, tant qu'elle n'est pas déjà utilisée pour une région de mémoire différente. Définissons-le comme la mémoire commençant à l'adresse 0x_4444_4444_0000afin que nous puissions facilement reconnaître un pointeur de tas plus tard
pub const HEAP_START: usize = 0x_4444_4444_0000;
//Nous avons défini la taille du tas sur 100 Kio pour l'instant. Si nous avons besoin de plus d'espace à l'avenir, nous pouvons simplement l'augmenter.
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called")
    }
}

//pour les implementation d'allocator ex allocator/bump
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

