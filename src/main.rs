//notre crate relie la bilbiothèque standard implicitement. Désactivons cela en ajoutant l’attribut no std
#![no_std]
//Pour indiquer au compilateur que nous ne voulons pas utiliser la chaîne de point d’entrée normale
#![no_main]

use core::panic::PanicInfo;

static HELLO: &[u8] = b"Hello World!";

//nous réécrivons le point d’entrée du système d’exploitation avec notre propre fonction _start
//En utilisant l’attribut #[no_mangle], nous désactivons la décoration de nom pour assurer que le compilateur Rust crée une fonction avec le nom _start. Sans cet attribut, le compilateur génèrerait un symbol obscure _ZN3blog_os4_start7hb173fedf945531caE pour donner un nom unique à chaque fonction.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // cette fonction est le point d'entrée, comme le linker cherche une fonction
    // nomée `_start` par défaut

    //le tampon VGA est situé à l'adresse 0xb8000
    //nous transformons l'entier 0xb8000en un pointeur brut
    let vga_buffer = 0xb8000 as *mut u8;
    // Ensuite, nous parcourons les octets de la chaîne d'octets statique .
    for (i, &byte) in HELLO.iter().enumerate() {
        //il y a un unsafe bloc autour de toutes les écritures en mémoire. La raison en est que le compilateur Rust ne peut pas prouver que les pointeurs bruts que nous créons sont valides
        //En les mettant dans un unsafebloc, nous disons essentiellement au compilateur que nous sommes absolument sûrs que les opérations sont valides
        unsafe {
            // nous utilisons la méthode pour écrire l'octet de chaîne et l'octet de couleur correspondant ( est un cyan clair)
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }

    loop {}
}

//L’attribut panic_handler définit la fonction que le compilateur doit appeler lorsqu’un panic arrive.
/// Cette fonction est appelée à chaque panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    //Le paramètre PanicInfo contient le fichier et la ligne où le panic a eu lieu et le message optionnel de panic.
    loop {}
}
