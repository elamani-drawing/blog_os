//notre crate relie la bilbiothèque standard implicitement. Désactivons cela en ajoutant l’attribut no std
#![no_std]
//Pour indiquer au compilateur que nous ne voulons pas utiliser la chaîne de point d’entrée normale
#![no_main]
//pour les tests
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
//Nous définissons le nom de la fonction d'entrée du framework de test sur test_main et allons l'appeller depuis notre _startpoint d'entrée.
#![reexport_test_harness_main = "test_main"]

use blog_os::println;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
//Pour s'assurer que la fonction de point d'entrée a toujours la signature correcte attendue par le chargeur de démarrage, le bootloadercrate fournit une entry_pointmacro qui fournit un moyen vérifié de type pour définir une fonction Rust comme point d'entrée.
entry_point!(kernel_main);
//Nous n'avons plus besoin d'utiliser extern "C"ou no_manglepour notre point d'entrée, car la macro définit _startpour nous le véritable point d'entrée de niveau inférieur
//kernel_mainfonction est maintenant une fonction Rust tout à fait normale, nous pouvons donc lui choisir un nom arbitraire.
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use blog_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::{structures::paging::Page, VirtAddr};

    println!("Hello World{}", "!");
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // map an unused page
    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    blog_os::hlt_loop();
}

// L’attribut panic_handler définit la fonction que le compilateur doit appeler lorsqu’un panic arrive.
// Cette fonction est appelée à chaque panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    //Le paramètre PanicInfo contient le fichier et la ligne où le panic a eu lieu et le message optionnel de panic.
    println!("{}", info);
    blog_os::hlt_loop();
}

//utilisation de la panic test
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
