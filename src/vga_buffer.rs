
//buffer
use volatile::Volatile;
//macros de formatage
use core::fmt;
use spin::Mutex;

//interface globale
//Pour fournir un écrivain global qui peut être utilisé comme interface à partir d'autres modules sans transporter d' Writerinstance, nous essayons de créer un static WRITER

use lazy_static::lazy_static;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

//couleur
//Normalement, le compilateur émettrait un avertissement pour chaque variante inutilisée. En utilisant l' #[allow(dead_code)]attribut, nous désactivons ces avertissements pour l' Colorénumération.
#[allow(dead_code)]
//En dérivant les traits Copy, Clone, Debug, PartialEqet Eq, nous activons la sémantique de copie pour le type et le rendons imprimable et comparable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
//En raison de l' repr(u8)attribut, chaque variante d'énumération est stockée en tant que fichier u8. En fait, 4 bits seraient suffisants, mais Rust n'a pas de u4type.
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}


//La ColorCodestructure contient l'octet de couleur complète, contenant la couleur de premier plan et d'arrière-plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
//Pour nous assurer que le ColorCodea exactement la même disposition de données qu'un u8
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

//tampon de texte
//Nous pouvons maintenant ajouter des structures pour représenter un caractère d'écran et le tampon de texte

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
//repr(C) il garantit que les champs de la structure sont disposés exactement comme dans une structure C et garantit ainsi le bon ordre des champs
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

//Au lieu d'un ScreenChar, nous utilisons maintenant un Volatile<ScreenChar>. (Le Volatiletype est générique et peut envelopper (presque) n'importe quel type). Cela garantit que nous ne pouvons pas y écrire accidentellement via une écriture "normale".
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}


//Pour écrire réellement à l'écran, nous créons maintenant un type d'écrivai
//Le rédacteur écrira toujours jusqu'à la dernière ligne et décalera les lignes vers le haut lorsqu'une ligne est pleine (ou le \n).
pub struct Writer {
    // Le column_positionchamp conserve la trace de la position actuelle dans la dernière ligne. 
    column_position: usize,
    color_code: ColorCode,
    //une référence au tampon VGA est stockée dans buffer
    // La 'staticdurée de vie spécifie que la référence est valide pour toute la durée d'exécution du programme (ce qui est vrai pour le tampon de texte VGA).
    buffer: &'static mut Buffer,
}


impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            //Si l'octet est l' octet de saut de ligne\n , le rédacteur n'imprime rien. Au lieu de cela, il appelle une new_lineméthode
            b'\n' => self.new_line(),
            byte => {
                //Lors de l'impression d'un octet, le rédacteur vérifie si la ligne courante est pleine. Dans ce cas, un new_lineappel est nécessaire avant de boucler la ligne
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                //Ensuite, il écrit un nouveau ScreenChardans le tampon à la position actuelle. Enfin, la position actuelle de la colonne est avancée.
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                //Au lieu d'une affectation normale utilisant =, nous utilisons maintenant la writeméthode . Cela garantit que le compilateur n'optimisera jamais cette écriture.
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    //Pour imprimer des chaînes entières, nous pouvons les convertir en octets et les imprimer un par un 
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            //Le tampon de texte VGA prend uniquement en charge ASCII et les octets supplémentaires de la page de codes 437 . Les chaînes de rouille sont UTF-8 par défaut, elles peuvent donc contenir des octets qui ne sont pas pris en charge par le tampon de texte VGA.
            //Nous utilisons une correspondance pour différencier les octets ASCII imprimables (une nouvelle ligne ou tout ce qui se trouve entre un espace et un ~caractère) et les octets non imprimables. Pour les octets non imprimables, nous imprimons un ■caractère, qui a le code hexadécimal 0xfesur le matériel VGA.
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }

        }
    }

    fn new_line(&mut self) {
        //Nous parcourons tous les caractères de l'écran et déplaçons chaque caractère d'une ligne vers le haut
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }
    //Cette méthode efface une ligne en remplaçant tous ses caractères par un espace.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

}

//utilisation des macros de formattage de rust
// pour  facilement imprimer différents types comme des entiers ou des flottants
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
