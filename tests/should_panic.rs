#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]
//Bien que nous ne puissions pas utiliser l' #[should_panic]attribut dans notre noyau, nous pouvons obtenir un comportement similaire en créant un test d'intégration qui se termine avec un code d'erreur de réussite du gestionnaire de panique
use core::panic::PanicInfo;

use blog_os::{QemuExitCode, exit_qemu, serial_println};
use blog_os::serial_print;


#[no_mangle]
pub extern "C" fn _start() -> ! {
    //cargo test --test should_panic
    //test_main();
    //parcequon a desactiver harness dans cargo.toml
    should_fail();
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

//Au lieu de réutiliser le test_runnerfrom our lib.rs, le test définit sa propre test_runnerfonction qui se termine avec un code de sortie d'échec lorsqu'un test revient sans paniquer (nous voulons que nos tests paniquent). Si aucune fonction de test n'est définie, le programme d'exécution se termine avec un code d'erreur de réussite. 
/*
//parcequ'on a desactiver harness dans cargo.toml
pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
        serial_println!("[test did not panic]");
        exit_qemu(QemuExitCode::Failed);
    }
    exit_qemu(QemuExitCode::Success);
}
*/

//Le test utilise le assert_eqpour affirmer que 0et 1sont égaux. Cela échoue bien sûr, de sorte que notre test panique comme souhaité. Notez que nous devons imprimer manuellement le nom de la fonction en utilisant serial_print!ici car nous n'utilisons pas le Testabletrait.
//#[test_case]//parcequ'on a desactiver harness dans cargo.toml
fn should_fail() {
    serial_print!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}
