use std::collections::HashMap;

/*
Bindings:

                            Keypad                   Keyboard
                            +-+-+-+-+                +-+-+-+-+
                            |1|2|3|C|                |1|2|3|4|
                            +-+-+-+-+                +-+-+-+-+
                            |4|5|6|D|                |Q|W|E|R|
                            +-+-+-+-+       =>       +-+-+-+-+
                            |7|8|9|E|                |A|S|D|F|
                            +-+-+-+-+                +-+-+-+-+
                            |A|0|B|F|                |Z|X|C|V|
                            +-+-+-+-+                +-+-+-+-+

 */
const KEY_BINDINGS: [(u8, char); 16] = [
    (0x0, 'X'),
    (0x1, '1'),
    (0x2, '2'),
    (0x3, '3'),
    (0x4, 'Q'),
    (0x5, 'W'),
    (0x6, 'E'),
    (0x7, 'A'),
    (0x8, 'S'),
    (0x9, 'D'),
    (0xA, 'Z'),
    (0xB, 'C'),
    (0xC, '4'),
    (0xD, 'R'),
    (0xE, 'F'),
    (0xF, 'V')
];

pub struct Keypad {
    mapping: HashMap<u8, char>,
    last_pressed_key: Option<u8>
}

impl Keypad {
    pub fn default() -> Self {
        let mut mapping = HashMap::new();

        for (hex_key, key_binding) in KEY_BINDINGS.iter() {
            mapping.insert(*hex_key, *key_binding);
        }
        Keypad {
            mapping,
            last_pressed_key: None
        }
    }

    pub fn keypress(&mut self, key: u8) {
        self.last_pressed_key = Some(key);
    }

    pub fn was_key_pressed(&self, key: u8) -> bool {
        self.last_pressed_key == Some(key)
    }

    pub fn any_key_pressed(&self) -> bool {
        self.last_pressed_key.is_some()
    }

    pub fn take_keypress(&mut self) -> Option<u8> {
        self.last_pressed_key.take()
    }
}