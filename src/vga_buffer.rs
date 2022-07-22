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

#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
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
                self.buffer.chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code,
                };
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

    fn new_line(&mut self) {/* TODO */}

}

// Pour écrire des caractères à l'écran, vous pouvez créer une fonction temporaire :
pub fn print_something() {
    //l crée d'abord un nouveau Writer qui pointe vers le tampon VGA à 0xb8000
    // tout d'abord, nous transformons l'entier en un pointeur brut0xb8000 mutable 
    // Ensuite, nous le convertissons en une référence mutable en le déréférencant (via ) et en l'empruntant immédiatement à nouveau (via ). Cette conversion nécessite un block , car le compilateur ne peut pas garantir que le pointeur brut est valide.*&mutunsafe
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_byte(b'H');
    writer.write_string("ello ");
    writer.write_string("Wörld!");
}