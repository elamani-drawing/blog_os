
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
//Au lieu de réimplémenter le lanceur de test, nous utilisons la test_runnerfonction de notre bibliothèque en changeant l' #![test_runner(crate::test_runner)]attribut en #![test_runner(blog_os::test_runner)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
//Étant donné que les tests d'intégration sont des exécutables distincts, nous devons fournir à nouveau tous les attributs de caisse ( no_std, no_main, test_runner, etc.)

//La convention pour les tests d'intégration dans Rust est de les placer dans un testsrépertoire à la racine du projet (c'est-à-dire à côté du srcrépertoire). Le framework de test par défaut et les frameworks de test personnalisés récupèrent et exécutent automatiquement tous les tests de ce répertoire.
//Tous les tests d'intégration sont leurs propres exécutables et complètement séparés de nos fichiers main.rs. Cela signifie que chaque test doit définir sa propre fonction de point d'entrée. Créons un exemple de test d'intégration nommé basic_bootpour voir comment cela fonctionne

use core::panic::PanicInfo;
use blog_os::println;

// Nous devons également créer une nouvelle fonction de point d'entrée _start, qui appelle la fonction de point d'entrée de test test_main. Nous n'avons besoin d'aucun cfg(test)attribut car les exécutables de test d'intégration ne sont jamais créés en mode non-test.
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    println!("test_println output");
}