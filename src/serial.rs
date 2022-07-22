
use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

//La uart_16550caisse contient une SerialPortstructure qui représente les registres UART, mais nous devons toujours en construire une instance nous-mêmes.
lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

//Comme l' isa-debug-exitappareil, l'UART est programmé à l'aide d'E/S de port. Étant donné que l'UART est plus complexe, il utilise plusieurs ports d'E/S pour programmer différents registres de périphériques. La fonction unsafe SerialPort::newattend l'adresse du premier port d'E/S de l'UART comme argument, à partir duquel elle peut calculer les adresses de tous les ports nécessaires. Nous transmettons l'adresse du port 0x3F8, qui est le numéro de port standard de la première interface série.
//Pour rendre le port série facilement utilisable, nous ajoutons serial_print!et serial_println!macros :

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}