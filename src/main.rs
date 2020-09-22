use std::path::Path;
use std::fs;
use std::time::{Duration, Instant};
use rand::Rng;
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

fn main() {
    let mut my_chip = Chip8::new();
    my_chip.load_font();
    my_chip.load_game(Path::new("/home/jf/Documents/chip8/roms/Brix [Andreas Gustafsson, 1990].ch8"));


    let mut window = Window::new(
        "CHIP-8",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });


    let mut cpu_timer = Instant::now();
    let mut delay_and_sound = Instant::now();


    // 4150 is an arbitrary choice
    window.limit_update_rate(Some(std::time::Duration::from_micros(4150)));
    while window.is_open() && !window.is_key_down(Key::Escape) {

        // 2ms => 500hz
        if cpu_timer.elapsed().subsec_millis() > 2 {
            my_chip.emulate_cycle(&window);
            cpu_timer = Instant::now();
        }

        // both timers run at 60hz
        if delay_and_sound.elapsed().as_micros() > 16667  {
            if my_chip.delay_timer > 0 {
                my_chip.delay_timer -= 1;

            }
            if my_chip.sound_timer > 0 {
                my_chip.sound_timer -= 1;

            }
            delay_and_sound = Instant::now();
        }

        window
            .update_with_buffer(&my_chip.display, WIDTH, HEIGHT)
            .unwrap();



    }


}


fn num_to_key(num: u8) -> Key {
    match num {
        0x0 => Key::Key1,
        0x1 => Key::Key2,
        0x2 => Key::Key3,
        0x3 => Key::Key4,
        0x4 => Key::Q,
        0x5 => Key::W,
        0x6 => Key::E,
        0x7 => Key::R,
        0x8 => Key::A,
        0x9 => Key::S,
        0xA => Key::D,
        0xB => Key::F,
        0xC => Key::Z,
        0xD => Key::X,
        0xE => Key::C,
        0xF => Key::V,
        _ => Key::Key1,
    }
}

fn key_to_num(key: Key) -> u8 {
    match key {
        Key::Key1 => 0x0,
        Key::Key2 => 0x1,
        Key::Key3 => 0x2,
        Key::Key4 => 0x3,
        Key::Q => 0x4,
        Key::W => 0x5,
        Key::E => 0x6,
        Key::R => 0x7,
        Key::A => 0x8,
        Key::S => 0x9,
        Key::D => 0xA,
        Key::F => 0xB,
        Key::Z => 0xC,
        Key::X => 0xD,
        Key::C => 0xE,
        Key::V => 0xF,
        _ => 0x0,
    }
}

fn nth_digit(number: u8, n: u16) -> u8 {
    number / 10u8.pow(n as u32) % 10
}


#[derive(Debug)]
struct Chip8{
    pc: u16,
    index: u16,
    sp: u8,
    display: Vec<u32>,
    stack: Vec<u16>,
    registers: Vec<u8>,
    memory: Vec<u8>,
    delay_timer: u8,
    sound_timer: u8,
}

impl Chip8 {
    fn new() -> Chip8 {
        Chip8 {
            pc: 0x200,
            index: 0,
            sp: 0,
            display: vec![0; 2048],
            stack: vec![0;16],
            registers: vec![0;16],
            memory: vec![0;4096],
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    fn load_font(&mut self) {
        for i in 0..80 {
            self.memory[i] = FONT[i];
        }

    }

    fn load_game(&mut self, path: &Path) {
        let buffer: Vec<u8> = fs::read(path).unwrap();

        for i in 0..buffer.len() {
            self.memory[0x200 + i] = buffer[i];
        }
    }

    fn emulate_cycle(&mut self, window: &Window) {
        let opcode: u16 = (self.memory[self.pc as usize] as u16) << 8 | self.memory[(self.pc + 1) as usize] as u16;

        //println!("{:?}", self.display);

        // split opcode into four four bits for convenience
        let a = ((opcode & 0xF000) >> 12) as usize;
        let b = ((opcode & 0x0F00) >> 8) as usize;
        let c = ((opcode & 0x00F0) >> 4) as usize;
        let d = (opcode & 0x000F) as usize;

        let cd = (opcode & 0x00FF) as u8;

        let bcd = (opcode & 0x0FFF) as u16;

        match opcode & 0xF000{
            0x0000 => {
                match opcode {
                    //00E0
                    0x00E0 => {
                        self.display = vec![0;WIDTH*HEIGHT];
                        self.pc += 2;
                    },
                    //00E0
                    0x00EE => {
                        self.pc = self.stack[self.sp as usize];
                        self.sp -= 1;
                        self.pc += 2;
                    },
                    _ => {}
                }

            },
            //1NNN
            0x1000 => {
                self.pc = bcd;
            },
            //2NNN
            0x2000 => {
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = bcd;
            },
            //3XKK
            0x3000 => {
                if self.registers[b] == cd { 
                    self.pc += 2;
                }
                self.pc += 2;
            },
            //4XKK
            0x4000 => {
                if self.registers[b] != cd{ 
                    self.pc += 2;
                }
                self.pc += 2;
            },
            //5XY0
            0x5000 => {
                if self.registers[b] == self.registers[c] {
                    self.pc += 2;
                }
                self.pc += 2;
            },
            //6XKK
            0x6000 => {
                self.registers[b] = cd;
                self.pc += 2;
            },
            //7XKK
            0x7000 => {
                let num = self.registers[b] as u16 + cd as u16;
                self.registers[b] = num as u8;
                self.pc += 2;
            }
            0x8000 => {
                match opcode & 0x000F {
                    //8XY0
                    0x0000 => {
                        self.registers[b] = self.registers[c];
                        self.pc += 2;
                    },
                    //8XY1
                    0x0001 => {
                        self.registers[b] = self.registers[b] | self.registers[c];
                        self.pc += 2;
                    },
                    //8XY2
                    0x0002 => {
                        self.registers[b] = self.registers[b] & self.registers[c];
                        self.pc += 2;
                    },
                    //8XY3
                    0x0003 => {
                        self.registers[b] = self.registers[b] ^ self.registers[c];
                        self.pc += 2;
                    },
                    //8XY4
                    0x0004 => {
                        let sum: u16 = self.registers[b] as u16 + self.registers[c] as u16;
                        if sum > 255 {
                            self.registers[0xF] = 1;
                        }
                        else {
                            self.registers[0xF] = 0;
                        }

                        self.registers[b] = sum as u8;
                        self.pc += 2;
                    },
                    //8XY5
                    0x0005 => {
                        if self.registers[b] > self.registers[c] {
                            self.registers[0xF] = 1;
                        }
                        else {
                            self.registers[0xF] = 0;
                        }
                        let num = self.registers[b] as i16 - self.registers[c] as i16;
                        self.registers[b] = num as u8;
                        self.pc += 2;

                    },
                    //8XY6
                    0x0006 => {
                        if self.registers[b] & 1 == 1 {
                            self.registers[0xF] = 1;
                        }
                        else {
                            self.registers[0xF] = 0;
                        }

                        self.registers[b] = self.registers[b] >> 1;
                        self.pc += 2;
                    },
                    //8XY7
                    0x0007 => {
                        if self.registers[c] > self.registers[b] {
                            self.registers[0xF] = 1;
                        }
                        else {
                            self.registers[0xF] = 0;
                        }
                        self.registers[b] = self.registers[c] - self.registers[b];
                        self.pc += 2;
                    },
                    //8XYE
                    0x000E => {
                        if self.registers[b] & 128 == 1 {
                            self.registers[0xF] = 1;
                        }
                        else {
                            self.registers[0xF] = 0;
                        }
                        self.registers[b] =  self.registers[b] << 1;
                        self.pc += 2;

                    },
                    _ => {}
                }

            },
            //9XY0
            0x9000 => {
                if self.registers[b] != self.registers[c] {
                    self.pc += 2;
                }
                self.pc += 2;
            },
            //ANNN
            0xA000 => {
                self.index = bcd;
                self.pc += 2;
            },
            //BNNN
            0xB000 => {
                self.pc = bcd + self.registers[0] as u16;
            },
            //CXKK
            0xC000 => {
                let random_numer: u8 = rand::thread_rng().gen();
                self.registers[b] = random_numer & cd;
                self.pc += 2;
            },
            //DXYN
            0xD000 => {
                let mut x = self.registers[b] as u16;
                let mut y = self.registers[c] as u16;

                self.registers[0xF] = 0;

                for row in 0..d {
                    // not sure about the x
                    if y > 31 || x > 63 {
                        break;
                    }
                    let byte = self.memory[self.index as usize + row];
                    for pos in (0..8).rev() {
                        let bit = (byte >> pos) & 1;
                        //println!("{}",x);
                        //println!("{}",y);

                        if bit==1 {
                            if self.display[(x + (y*64)) as usize]==0x00FFFFFF {
                                self.display[(x+(y*64)) as usize] = 0;
                                self.registers[0xF] = 1;
                            }
                            else {
                                self.display[(x+(y*64)) as usize] = 0x00FFFFFF;
                            }
                        }

                        x += 1;

                        if x % 64 == 0 {
                            break
                        }
                    }
                    x = self.registers[b] as u16;
                    y += 1;

                }
                self.pc += 2;

            },
            0xE000 => {
                match opcode & 0x00FF {
                    //EX9E
                    0x009E => {
                        if window.is_key_down(num_to_key(self.registers[b])) {
                            self.pc += 2;
                        }
                        self.pc += 2;
                    },
                    //EXA1
                    0x00A1 => {
                        if !window.is_key_down(num_to_key(self.registers[b])) {
                            self.pc += 2;
                        }
                        self.pc += 2;
                    },
                    _ => {}
                }

            },
            0xF000 => {
                match opcode & 0x00FF {
                    //FX07
                    0x0007 => {
                        self.registers[b] = self.delay_timer;
                        self.pc += 2;
                    },
                    //FX0A
                    0x000A => {
                        let keys = window.get_keys().unwrap();


                        if keys.len() != 0 {
                            self.registers[b] = key_to_num(keys[0]);
                            self.pc += 2;
                        }

                    },
                    //FX15
                    0x0015 => {
                        self.delay_timer = self.registers[b];
                        self.pc += 2;
                    },
                    //FX18
                    0x0018 => {
                        self.sound_timer = self.registers[b];
                        self.pc += 2;
                    },
                    //FX1E
                    0x001E => {
                        self.index += self.registers[b] as u16;
                        self.pc += 2;
                    },
                    //FX29
                    0x0029 => {
                        self.index = (self.registers[b] as u16) * 5;
                        self.pc += 2;
                    },
                    //FX33
                    0x0033 => {
                        let v = self.registers[b];
                        self.memory[self.index as usize] = nth_digit(v,2);
                        self.memory[self.index as usize + 1] = nth_digit(v,1);
                        self.memory[self.index as usize + 2] = nth_digit(v,0);
                        self.pc += 2;

                    },
                    //FX55
                    0x0055 => {

                        for i in 0..(b+1) {
                            self.memory[self.index as usize + i] = self.registers[i];
                        }
                        self.pc += 2;

                    },
                    //FX65
                    0x0065 => {
                        for i in 0..(b+1) {
                            self.registers[i] = self.memory[self.index as usize + i];
                        }
                        self.pc += 2;

                    },
                    _ => {}
                }
            },
            _ => {}
        }

    }
}


