use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Emu {
    pc: u16,             // program counter
    ram: [u8; RAM_SIZE], // RAM
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16, // Stack pointer
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8, // Delay timer
    st: u8, // Sound timer
}

const START_ADDR: u16 = 0x200;

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        return new_emu;
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();
        // Decode and execute
        self.execute(op);
        self.pc += 2;
    }

    pub fn tick_timers(&mut self) {
        self.dt = if self.dt > 0 { self.dt - 1 } else { 0 };

        if self.st > 0 {
            if self.st == 1 {
                // BEEP
            }
            self.st -= 1;
        }
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        if idx > self.keys.len() - 1 {
            return;
        }
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => self.op_0000(),
            (0, 0, 0xE, 0) => self.op_00E0(),
            (0, 0, 0xE, 0xE) => self.op_00EE(),
            (1, _, _, _) => self.op_1NNN(op),
            (2, _, _, _) => self.op_2NNN(op),
            (3, _, _, _) => self.op_3XNN(op),
            (4, _, _, _) => self.op_4XNN(op),
            (5, _, _, 0) => self.op_5XY0(op),
            (6, _, _, _) => self.op_6XNN(op),
            (7, _, _, _) => self.op_7XNN(op),
            (8, _, _, 0) => self.op_8XY0(op),
            (8, _, _, 1) => self.op_8XY1(op),
            (8, _, _, 2) => self.op_8XY2(op),
            (8, _, _, 3) => self.op_8XY3(op),
            (8, _, _, 4) => self.op_8XY4(op),
            (8, _, _, 5) => self.op_8XY5(op),
            (8, _, _, 6) => self.op_8XY6(op),
            (8, _, _, 7) => self.op_8XY7(op),
            (8, _, _, 0xE) => self.op_8XYE(op),
            (9, _, _, 0) => self.op_9XY0(op),
            (0xA, _, _, _) => self.op_ANNN(op),
            (0xB, _, _, _) => self.op_BNNN(op),
            (0xC, _, _, _) => self.op_CXNN(op),
            (0xD, _, _, _) => self.op_DXYN(op),
            (0xE, _, 9, 0xE) => self.op_EX9E(op),
            (0xE, _, 0xA, 1) => self.op_EXA1(op),
            (0xF, _, 0, 7) => self.op_FX07(op),
            (0xF, _, 0, 0xA) => self.op_FX0A(op),
            (0xF, _, 1, 5) => self.op_FX15(op),
            (0xF, _, 1, 8) => self.op_FX18(op),
            (0xF, _, 1, 0xE) => self.op_FX1E(op),
            (0xF, _, 2, 9) => self.op_FX29(op),
            (0xF, _, 3, 3) => self.op_FX33(op),
            (0xF, _, 5, 5) => self.op_FX55(op),
            (0xF, _, 6, 5) => self.op_FX65(op),
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }

    fn fetch(&mut self) -> u16 {
        let high_byte = self.ram[self.pc as usize] as u16;
        let low_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (high_byte << 8) | low_byte;
        self.pc += 2;
        op
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    // NOP
    fn op_0000(&mut self) {
        return;
    }

    // Clear screen
    fn op_00E0(&mut self) {
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
    }

    fn op_00EE(&mut self) {
        let return_address = self.pop();
        self.pc = return_address;
    }

    fn op_1NNN(&mut self, op: u16) {
        let address = op & 0x0FFF;
        self.pc = address;
    }

    fn op_2NNN(&mut self, op: u16) {
        self.push(self.pc);
        let address = op & 0x0FFF;
        self.pc = address;
    }

    fn op_3XNN(&mut self, op: u16) {
        let reg = ((op & 0x0F00) >> 8) as usize;
        let nn = (op & 0x00FF) as u8;
        if self.v_reg[reg] == nn {
            self.pc += 2;
        }
    }

    fn op_4XNN(&mut self, op: u16) {
        let reg = ((op & 0x0F00) >> 8) as usize;
        let nn = (op & 0x00FF) as u8;

        if self.v_reg[reg] != nn {
            self.pc += 2;
        }
    }

    fn op_5XY0(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        let regy = ((op & 0x00F0) >> 4) as usize;

        if self.v_reg[regx] == self.v_reg[regy] {
            self.pc += 2;
        }
    }

    fn op_6XNN(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        let nn = (op & 0x00FF) as u8;
        self.v_reg[regx] = nn;
    }

    fn op_7XNN(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        let nn = (op & 0x00FF) as u8;

        self.v_reg[regx] += nn;
    }

    fn op_8XY0(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        let regy = ((op & 0x00F0) >> 4) as usize;

        self.v_reg[regx] = self.v_reg[regy];
    }

    fn op_8XY1(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        let regy = ((op & 0x00F0) >> 4) as usize;

        self.v_reg[regx] |= self.v_reg[regy];
    }
    fn op_8XY2(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        let regy = ((op & 0x00F0) >> 4) as usize;

        self.v_reg[regx] &= self.v_reg[regy];
    }
    fn op_8XY3(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        let regy = ((op & 0x00F0) >> 4) as usize;

        self.v_reg[regx] ^= self.v_reg[regy];
    }
    fn op_8XY4(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        let regy = ((op & 0x00F0) >> 4) as usize;

        let (vx, carry) = self.v_reg[regx].overflowing_add(self.v_reg[regy]);
        let vf = if carry { 1 } else { 0 };

        self.v_reg[regx] = vx;
        self.v_reg[0xF] = vf;
    }
    fn op_8XY5(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        let regy = ((op & 0x00F0) >> 4) as usize;

        let (vx, borrow) = self.v_reg[regx].overflowing_sub(self.v_reg[regy]);
        let vf = if borrow { 0 } else { 1 };

        self.v_reg[regx] = vx;
        self.v_reg[0xF] = vf;
    }
    fn op_8XY6(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;

        let lsb = self.v_reg[regx] & 1;
        self.v_reg[regx] >>= 1;
        self.v_reg[0xF] = lsb;
    }
    fn op_8XY7(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        let regy = ((op & 0x00F0) >> 4) as usize;

        let (vx, borrow) = self.v_reg[regy].overflowing_sub(self.v_reg[regx]);
        let vf = if borrow { 0 } else { 1 };

        self.v_reg[regx] = vx;
        self.v_reg[0xF] = vf;
    }
    fn op_8XYE(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;

        let msb = (self.v_reg[regx] >> 7) & 1;
        self.v_reg[regx] <<= 1;
        self.v_reg[0xF] = msb;
    }

    fn op_9XY0(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        let regy = ((op & 0x00F0) >> 4) as usize;

        if self.v_reg[regx] != self.v_reg[regy] {
            self.pc += 2;
        }
    }

    fn op_ANNN(&mut self, op: u16) {
        let nnn = op & 0x0FFF;
        self.i_reg = nnn;
    }

    fn op_BNNN(&mut self, op: u16) {
        let nnn = op & 0x0FFF;
        self.pc = (self.v_reg[0] as u16) + nnn;
    }

    fn op_CXNN(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        let nn = (op & 0x00FF) as u8;
        let rng: u8 = random();
        self.v_reg[regx] = rng & nn;
    }

    fn op_DXYN(&mut self, op: u16) {
        let x = self.v_reg[((op & 0x0F00) >> 8) as usize] as u16;
        let y = self.v_reg[((op & 0x00F0) >> 4) as usize] as u16;
        let rows = (op & 0x00F0) >> 4;
        let mut flipped = false;

        for row in 0..rows {
            let addr = self.i_reg + row;
            let pixels = self.ram[addr as usize];

            for column in 0..8 {
                if (pixels & (0b1000_0000 >> column)) != 0 {
                    let x = (x + column) as usize % SCREEN_WIDTH;
                    let y = (y + row) as usize % SCREEN_HEIGHT;

                    let idx = x + SCREEN_WIDTH * y;
                    flipped |= self.screen[idx];
                    self.screen[idx] = true; // TODO Check
                }
            }
        }

        self.v_reg[0xF] = if flipped { 1 } else { 0 };
    }

    fn op_EX9E(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        let vx = self.v_reg[regx] as usize;
        let key = self.keys[vx];

        if key {
            self.pc += 2;
        }
    }

    fn op_EXA1(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        let vx = self.v_reg[regx] as usize;
        let key = self.keys[vx];

        if !key {
            self.pc += 2;
        }
    }

    fn op_FX07(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        self.v_reg[regx] = self.dt;
    }

    fn op_FX0A(&mut self, op: u16) {
        // Wait for key
        let mut pressed = false;

        for i in 0..self.keys.len() {
            if self.keys[i] {
                pressed = true;
                break;
            }
        }

        if !pressed {
            self.pc -= 2;
        }
    }

    fn op_FX15(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        self.dt = self.v_reg[regx];
    }

    fn op_FX18(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        self.st = self.v_reg[regx];
    }

    fn op_FX1E(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        let vx = self.v_reg[regx] as u16;
        self.i_reg = self.i_reg.wrapping_add(vx);
    }

    fn op_FX29(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        let char = self.v_reg[regx] as u16;
        self.i_reg = char * 5;
    }

    fn op_FX33(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        let vx = self.v_reg[regx] as f32;
        let hundreds = (vx / 100.0).floor() as u8;
        let tens = ((vx / 10.0) % 10.0).floor() as u8;
        let ones = (vx % 10.0) as u8;

        self.ram[self.i_reg as usize] = hundreds;
        self.ram[self.i_reg as usize + 1] = tens;
        self.ram[self.i_reg as usize + 2] = ones;
    }

    fn op_FX55(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        for idx in 0..=regx {
            self.ram[self.i_reg as usize + idx] = self.v_reg[idx];
        }
    }

    fn op_FX65(&mut self, op: u16) {
        let regx = ((op & 0x0F00) >> 8) as usize;
        for idx in 0..=regx {
            self.v_reg[idx] = self.ram[self.i_reg as usize + idx]
        }
    }
}
