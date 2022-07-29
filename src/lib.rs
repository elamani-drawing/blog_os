#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

pub mod serial;
pub mod vga_buffer;

//pour les exceptions

pub mod interrupts;
pub mod gdt;

pub fn init() {
    interrupts::init_idt();
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


//Notre exécuteur imprime simplement un court message de débogage, puis appelle chaque fonction de test de la liste. 
//Le type d'argument &[&dyn Fn()]est une tranche de références d' objet de trait du trait Fn() . Il s'agit essentiellement d'une liste de références à des types qui peuvent être appelés comme une fonction.
//#[cfg(test)]
//Pour rendre notre test_runnerdisponible aux exécutables et aux tests d'intégration, nous ne lui appliquons pas l' cfg(test)attribut et le rendons public
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    //nous mettons a jou rnotre test runner pour quitter qemu apres l'execution de tous les tests
    exit_qemu(QemuExitCode::Success);
}



// Pour quitter QEMU avec un message d'erreur sur une panique, nous pouvons utiliser la compilation conditionnelle pour utiliser un gestionnaire de panique différent en mode test
//Nous factorisons également l'implémentation de notre gestionnaire de panique dans une test_panic_handlerfonction publique, afin qu'il soit également disponible pour les exécutables.
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
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



/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}