
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
//Nous réutilisons les fonctions test_runneret test_panic_handlerde notre lib.rs. Puisque nous voulons tester les allocations, 
//nous activons la alloccaisse via la extern crate allocdéclaration. Pour plus d'informations sur le passe-partout de test, consultez le post de test .
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

use alloc::boxed::Box;
use alloc::vec::Vec;
use blog_os::allocator::HEAP_SIZE;

entry_point!(main);


//Elle est très similaire à la kernel_mainfonction de notre main.rs, avec les différences que nous n'invoquons pas println, n'incluons aucun exemple
// d'allocation et appelons test_mainsans condition.
fn main(boot_info: &'static BootInfo) -> ! {
    //cargo test --test heap_allocation
    use blog_os::allocator;
    use blog_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    blog_os::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

//ce test vérifie qu'aucune erreur d'allocation ne se produit.
#[test_case]
fn simple_allocation() {
    //Tout d'abord, nous ajoutons un test qui effectue quelques allocations simples en utilisant Boxet vérifie les valeurs allouées, pour s'assurer que les allocations de base fonctionnent :
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}

//Ensuite, nous construisons de manière itérative un grand vecteur, pour tester à la fois les grandes allocations et les allocations multiples (dues aux réallocations) :
#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    //Nous vérifions la somme en la comparant à la formule de la n-ième somme partielle . Cela nous donne une certaine confiance dans le fait que les valeurs attribuées sont toutes correctes.
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

//nous créons dix mille allocations les unes après les autres
//Ce test garantit que l'allocateur réutilise la mémoire libérée pour les allocations ultérieures, car sinon il manquerait de mémoire.
#[test_case]
fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}