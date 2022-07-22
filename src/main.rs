//notre crate relie la bilbiothèque standard implicitement. Désactivons cela en ajoutant l’attribut no std
#![no_std]
//Pour indiquer au compilateur que nous ne voulons pas utiliser la chaîne de point d’entrée normale
#![no_main]

use core::panic::PanicInfo;
mod vga_buffer;

static HELLO: &[u8] = b"Hello World!";

//nous réécrivons le point d’entrée du système d’exploitation avec notre propre fonction _start
//En utilisant l’attribut #[no_mangle], nous désactivons la décoration de nom pour assurer que le compilateur Rust crée une fonction avec le nom _start. Sans cet attribut, le compilateur génèrerait un symbol obscure _ZN3blog_os4_start7hb173fedf945531caE pour donner un nom unique à chaque fonction.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // cette fonction est le point d'entrée, comme le linker cherche une fonction
    // nomée `_start` par défaut

    vga_buffer::print_something();

    loop {}
}

//L’attribut panic_handler définit la fonction que le compilateur doit appeler lorsqu’un panic arrive.
/// Cette fonction est appelée à chaque panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    //Le paramètre PanicInfo contient le fichier et la ligne où le panic a eu lieu et le message optionnel de panic.
    loop {}
}
