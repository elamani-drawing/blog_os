//notre crate relie la bilbiothèque standard implicitement. Désactivons cela en ajoutant l’attribut no std
#![no_std]
//Pour indiquer au compilateur que nous ne voulons pas utiliser la chaîne de point d’entrée normale
#![no_main]
//pour les tests
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
//Nous définissons le nom de la fonction d'entrée du framework de test sur test_main et allons l'appeller depuis notre _startpoint d'entrée. 
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
mod vga_buffer;

mod serial;

//nous réécrivons le point d’entrée du système d’exploitation avec notre propre fonction _start
//En utilisant l’attribut #[no_mangle], nous désactivons la décoration de nom pour assurer que le compilateur Rust crée une fonction avec le nom _start. Sans cet attribut, le compilateur génèrerait un symbol obscure _ZN3blog_os4_start7hb173fedf945531caE pour donner un nom unique à chaque fonction.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // cette fonction est le point d'entrée, comme le linker cherche une fonction
    // nomée `_start` par défaut
   
    println!("Hello World{}", "!");
    //panic!("Some panic message");
    #[cfg(test)]
    test_main();
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

// Pour quitter QEMU avec un message d'erreur sur une panique, nous pouvons utiliser la compilation conditionnelle pour utiliser un gestionnaire de panique différent en mode test
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

//Notre exécuteur imprime simplement un court message de débogage, puis appelle chaque fonction de test de la liste. 
//Le type d'argument &[&dyn Fn()]est une tranche de références d' objet de trait du trait Fn() . Il s'agit essentiellement d'une liste de références à des types qui peuvent être appelés comme une fonction.
#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    //nous mettons a jou rnotre test runner pour quitter qemu apres l'execution de tous les tests
    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

//maintenant utiliser le Porttype fourni par le crate pour créer une exit_qemufonctio
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
//Pour spécifier le statut de sortie, nous créons une QemuExitCodeénumération. L'idée est de sortir avec le code de sortie succès si tous les tests ont réussi et avec le code de sortie échec sinon
pub enum QemuExitCode {
    //0x10 pour le succès et 0x11 pour l'échec
    Success = 0x10,
    Failed = 0x11,
}

//La fonction crée un nouveau Portat 0xf4, qui est le iobasede l' isa-debug-exitappareil. Ensuite, il écrit le code de sortie passé sur le port. Nous l'utilisons u32parce que nous avons spécifié iosizele isa-debug-exitpériphérique sur 4 octets. Les deux opérations ne sont pas sûres, car l'écriture sur un port d'E/S peut généralement entraîner un comportement arbitraire.
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

//L'ajout manuel de ces instructions d'impression pour chaque test que nous écrivons est fastidieux (serial_println!("[ok]"); etc.), alors mettons à jour notre test_runnerpour imprimer ces messages automatiquement. 
pub trait Testable {
    fn run(&self) -> ();
}
//L'astuce consiste maintenant à implémenter ce trait pour tous les types Tqui implémentent le Fn()trait 
impl<T> Testable for T
where
    T: Fn(),
{
    //Nous implémentons la runfonction en imprimant d'abord le nom de la fonction à l'aide de la any::type_namefonction
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();//nous invoquons la fonction de test via,  Cela ne fonctionne que parce que nous exigeons que selfimplémente le Fn()
        serial_println!("[ok]"); //Après le retour de la fonction de test, nous imprimons [ok]pour indiquer que la fonction n'a pas paniqué.
    }
}