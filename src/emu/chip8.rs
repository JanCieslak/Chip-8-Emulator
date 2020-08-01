use rand::Rng;

pub struct Chip8 {
    stack: [u16; 16],
    sp: u16,

    memory: [u8; 4096],
    v: [u8; 16],

    pc: u16,
    opcode: u16,
    i: u16,

    delay_timer: u8,
    sound_timer: u8,

    pub gfx: [u32; 64 * 32],
    pub key: [u8; 16],
    pub draw_flag: bool
}

impl Chip8 {
    pub fn new() -> Self {
        Chip8 {
            stack: [0; 16],
            sp: 0,

            memory: [0; 4096],
            v: [0; 16],

            pc: 0x0200,
            opcode: 0,
            i: 0,

            delay_timer: 0,
            sound_timer: 0,

            gfx: [0; 64 * 32],
            key: [0; 16],
            draw_flag: false
        }
    }

    fn emulate_cycle(&mut self) {
        self.opcode  = ((self.memory[self.pc as usize] as u16) << 8) | (self.memory[self.pc as usize + 1] as u16);
        let x   = (self.opcode | 0x0F00) >> 8;
        let y   = (self.opcode | 0x00F0) >> 4;
        let n   = self.opcode | 0x000F;
        let nn  = self.opcode | 0x00FF;
        let nnn = self.opcode | 0x0FFF;

        match self.opcode {
            0x0000 => {
                match self.opcode & 0x00FF {
                    0x00E0 => {
                        // clear the screen
                        for i in 0..(64 * 32) {
                            self.gfx[i] = 0;
                        }
                        self.draw_flag = true;
                        self.pc += 2;
                    },
                    0x00EE => {
                        // return from a subroutine
                        self.sp -= 1;
                        self.pc = self.stack[self.sp as usize];
                        self.pc += 2;
                    },
                    _ => panic!(format!("Unknown opcode (0x0000 branch): {}", self.opcode))
                }
            },
            0x1000 => {
                // jump to address nnn
                self.pc = nnn
            },
            0x2000 => {
                // calls subroutine at nnn
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            },
            0x3000 => {
                // skips next instruction if:
                if self.v[x as usize] == nn as u8 {
                    self.pc += 2;
                }
                self.pc += 2;
            },
            0x4000 => {
                // skips next instruction if:
                if self.v[x as usize] != nn as u8 {
                    self.pc += 2;
                }
                self.pc += 2;
            },
            0x5000 => {
                // skips next instruction if:
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc += 2;
                }
                self.pc += 2;
            },
            0x6000 => {
                // set VX to NN
                self.v[x as usize] = nn as u8;
                self.pc += 2;
            },
            0x7000 => {
                // add NN to VX
                self.v[x as usize] += nn as u8;
                self.pc += 2;
            },
            0x8000 => {
                match self.opcode & 0x000F {
                    0x0000 => {
                        // set VX to VY
                        self.v[x as usize] = self.v[y as usize];
                        self.pc += 2;
                    },
                    0x0001 => {
                        // set VX to VX | VY
                        self.v[x as usize] |= self.v[y as usize];
                        self.pc += 2;
                    },
                    0x0002 => {
                        // set VX to VX & VY
                        self.v[x as usize] &= self.v[y as usize];
                        self.pc += 2;
                    },
                    0x0003 => {
                        // set VX to VX ^ VY
                        self.v[x as usize] ^= self.v[y as usize];
                        self.pc += 2;
                    },
                    0x0004 => {
                        // add VY to VX, VF is set to 1 when there is a carry, else to 0
                        self.v[x as usize] += self.v[y as usize];
                        if self.v[x as usize] > (0xFF - self.v[y as usize]) {
                            self.v[0xF] = 1; // carry
                        } else {
                            self.v[0xF] = 0;
                        }
                        self.pc += 2;
                    },
                    0x0005 => {
                        // VF is set to 0 when there is a borrow, else to 1, sub VY from VX
                        if self.v[y as usize] > self.v[x as usize] {
                            self.v[0xF] = 0; // there is a borrow
                        } else {
                            self.v[0xF] = 1;
                        }
                        self.v[x as usize] = (self.v[x as usize] - self.v[y as usize]);
                        self.pc += 2;
                    },
                    0x0006 => {
                        // set VF with VX least significant bit and then shift VX to the right by 1
                        self.v[0xF] = (self.v[x as usize] & 1);
                        self.v[x as usize] >>= 1;
                        self.pc += 2;
                    },
                    0x0007 => {
                        // VF is set to 0 when there is a borrow, else to 1, set VX to VY sub VX
                        if self.v[x as usize] > self.v[y as usize] {
                            self.v[0xF] = 0; // there is a borrow
                        } else {
                            self.v[0xF] = 1;
                        }
                        self.v[x as usize] = (self.v[y as usize] - self.v[x as usize]);
                        self.pc += 2;
                    },
                    0x000E => {
                        // store the most significant bit of VX in VF and then shift VX to the left by 1
                        self.v[0x0F] = (self.v[x as usize] >> 7);
                        self.v[x as usize] <<= 1;
                        self.pc += 2;
                    },
                    _ => panic!(format!("Unknown opcode (0x8000 branch): {}", self.opcode))
                }
            },
            0x9000 => {
                // skips next instruction if:
                if self.v[x as usize] != self.v[y as usize] {
                    self.pc += 2;
                }
                self.pc += 2;
            },
            0xA000 => {
                // set i to NNN
                self.i = nnn;
                self.pc += 2;
            },
            0xB000 => {
                // jump to the address V0 + NNN
                self.pc = self.v[0] as u16 + nnn;
            },
            0xC000 => {
                // set VX to random(0 to 255) & NN
                self.v[x as usize] = (rand::thread_rng().gen_range(0, 256) & nn) as u8;
                self.pc += 2;
            },
            0xD000 => {
                // Draws a sprite at coordinate (VX, VY) that has a width of 8
                // pixels and a height of N pixels.
                // Each row of 8 pixels is read as bit-coded starting from memory
                // location I;
                // I value doesn't change after the execution of this instruction.
                // VF is set to 1 if any screen pixels are flipped from set to unset
                // when the sprite is drawn, and to 0 if that doesn't happen

                let height = n;
                let mut pixel;

                self.v[0xF] = 0;
                for y_line in 0..height {
                    pixel = self.memory[(self.i + y_line) as usize];
                    for x_line in 0..8 {
                        if (pixel & (0x80 >> x_line)) != 0 {
                            if self.gfx[(x + x_line as u16 + ((y + y_line) * 64)) as usize] == 1 {
                                self.v[0xF] = 1;
                            }
                            self.gfx[(x + x_line as u16 + ((y + y_line) * 64)) as usize] ^= 1;
                        }
                    }
                }

                self.draw_flag = true;
                self.pc += 2;
            },
            0xE000 => {
                match self.opcode & 0x00FF {
                    0x009E => {
                        // if (key[VX] != 0) skip next instruction
                        if self.key[self.v[x as usize] as usize] != 0 {
                            self.pc += 2;
                        }
                        self.pc += 2;
                    },
                    0x00A1 => {
                        // if (key[VX] == 0) skip next instruction
                        if self.key[self.v[x as usize] as usize] == 0 {
                            self.pc += 2;
                        }
                        self.pc += 2;
                    },
                    _ => panic!(format!("Unknown opcode (0xE000 branch): {}", self.opcode))
                }
            },
            0xF000 => {
                match self.opcode & 0x00FF {
                    0x0007 => {
                        // set VX to delay timer
                        self.v[x as usize] = self.delay_timer;
                        self.pc += 2;
                    },
                    0x000A => {
                        // await key press and store (for) i in VX
                        let mut key_pressed = false;

                        for i in 0..16 {
                            if self.key[i] != 0 {
                                self.v[x as usize] = i as u8;
                                key_pressed = true;
                                break;
                            }
                        }

                        if !key_pressed {
                            return;
                        }

                        self.pc += 2;
                    },
                    0x0015 => {
                        // set delay timer to VX
                        self.delay_timer = self.v[x as usize];
                        self.pc += 2;
                    },
                    0x0018 => {
                        // set sound timer to VX
                        self.sound_timer = self.v[x as usize];
                        self.pc += 2;
                    },
                    0x001E => {
                        // add VX to I, VF is set to 1 when there is range overflow (I + VX > 0xFFF), and 0 where there isn't
                        if self.i + self.v[x as usize] as u16 > 0x0FFF {
                            self.v[0xF] = 1;
                        } else {
                            self.v[0xF] = 0;
                        }
                        self.i += self.v[x as usize] as u16;
                        self.pc += 2;
                    },
                    0x0029 => {
                        // set I to the location of the sprite for the
                        // character in VX. Characters 0-F (in hexadecimal) are
                        // represented by a 4x5 font
                        self.i = (self.v[x as usize] * 5) as u16;
                        self.pc += 2;
                    },
                    0x0033 => {
                        // store the Binary-coded decimal representation of VX
                        // at the addresses I, I plus 1, and I plus 2
                        self.memory[self.i as usize] = self.v[x as usize] / 100;
                        self.memory[(self.i + 1) as usize] = (self.v[x as usize] / 10) % 10;
                        self.memory[(self.i + 2) as usize] = self.v[x as usize] % 10;
                        self.pc += 2;
                    },
                    0x0055 => {
                        // stores V0 to VX in memory starting at address I
                        for i in 0..=self.v[x as usize] {
                            self.memory[(self.i + i as u16) as usize] = self.v[i as usize];
                        }

                        // On the original interpreter, when the
                        // operation is done, I = I + X + 1.
                        self.i += (self.v[x as usize] + 1) as u16;

                        self.pc += 2;
                    },
                    0x0065 => {
                        for i in 0..=self.v[x as usize] {
                            self.v[i as usize] = self.memory[(self.i + i as u16) as usize];
                        }

                        // On the original interpreter, when the
                        // operation is done, I = I + X + 1.
                        self.i += (self.v[x as usize] + 1) as u16;

                        self.pc += 2;
                    },
                    _ => panic!(format!("Unknown opcode (0xF000 branch): {}", self.opcode))
                }
            },
            _ => panic!(format!("Unknown opcode: {}", self.opcode))
        }

        // Update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // todo sound
                println!("BEEP!");
            }
            self.sound_timer -= 1;
        }
    }
}