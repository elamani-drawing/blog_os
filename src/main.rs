//notre crate relie la bilbiothèque standard implicitement. Désactivons cela en ajoutant l’attribut no std
#![no_std]
//Pour indiquer au compilateur que nous ne voulons pas utiliser la chaîne de point d’entrée normale
#![no_main]
//pour les tests
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
//Nous définissons le nom de la fonction d'entrée du framework de test sur test_main et allons l'appeller depuis notre _startpoint d'entrée. 
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use blog_os::println;

//nous réécrivons le point d’entrée du système d’exploitation avec notre propre fonction _start
//En utilisant l’attribut #[no_mangle], nous désactivons la décoration de nom pour assurer que le compilateur Rust crée une fonction avec le nom _start. Sans cet attribut, le compilateur génèrerait un symbol obscure _ZN3blog_os4_start7hb173fedf945531caE pour donner un nom unique à chaque fonction.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // cette fonction est le point d'entrée, comme le linker cherche une fonction
    // nomée `_start` par défaut
   
    
    println!("Hello World{}", "!");

    blog_os::init(); // new

    // trigger a page fault
    unsafe {
        *(0xdeadbeef as *mut u64) = 42;
    };

    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
    
    //panic!("Some panic message");
    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    loop {}
}

// L’attribut panic_handler définit la fonction que le compilateur doit appeler lorsqu’un panic arrive.
// Cette fonction est appelée à chaque panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    //Le paramètre PanicInfo contient le fichier et la ligne où le panic a eu lieu et le message optionnel de panic.
    
    println!("{}", info);
    loop {}
}


#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

//utilisation de la panic test
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}