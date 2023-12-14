const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const PIXEL_ON: u8 = 0x01;
const PIXEL_OFF: u8 = 0x00;

pub struct Screen {
    screen: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
    collision: bool
}

impl Screen {
    pub fn default() -> Self {
        Screen {
            screen: [[PIXEL_OFF; SCREEN_WIDTH]; SCREEN_HEIGHT],
            collision: false
        }
    }

    pub fn draw_sprite_at_location(
        &mut self,
        pixel: u8,
        x_coord: u8,
        y_coord: u8,
    ) -> bool {
        for xline in 0..8 {
            // this loop will scan through each bit in pixel_value and extract its content
            // which is then used to determine if a collision occurred and will flag it in the register


            // value of a pixel at a specific location.
            let pixel_value = pixel & (0x80 >> xline);

            // TODO: for some reason, everyone checks if pixel_value != 0 and only then draws the pixel?
            if pixel_value != 0 {
                // check if collision occurred which occurs when a pixel changed
                // from 1 to 0 during a XOR operation.
                let collision = self.screen[y_coord as usize][(x_coord + xline) as usize] == PIXEL_ON;
                match collision {
                    true => self.flag_collision(),
                    false => ()
                }

                // draw pixel value at location now
                self.draw_pixel_at_location(
                    x_coord + xline,
                    y_coord,
                );
            }
        }
        let collision = self.collision.clone();
        self.reset_collision();
        collision
    }

    fn flag_collision(&mut self) {
        self.collision = true;
    }

    fn reset_collision(&mut self) {
        self.collision = false;
    }

    pub fn draw_pixel_at_location(&mut self, x: u8, y: u8) {
        self.screen[y as usize][x as usize] ^= PIXEL_ON;
    }

    pub fn how_many_ones(&self) -> u8 {
        let mut counter : u8= 0;
        for row in self.screen.iter() {
            for pixel_value in row.iter() {
                match pixel_value {
                    &PIXEL_ON => counter += 1,
                    _ => ()
                }
            }
        }
        counter
    }

    pub fn clear_screen(&mut self) {
        for row in self.screen.iter_mut() {
            for pixel_value in row.iter_mut() {
                *pixel_value = 0;
            }
        }
    }

}