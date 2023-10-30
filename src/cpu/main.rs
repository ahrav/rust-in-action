const MEMORY_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const REGISTER_SIZE: usize = 16;

// CPU emulates the CPU of the Chip-8 system.
// It contains the registers, memory, stack, stack pointer and program counter.
struct CPU {
    registers: [u8; REGISTER_SIZE],
    memory: [u8; MEMORY_SIZE],
    stack: [u16; STACK_SIZE],
    stack_pointer: u16,
    program_counter: usize,
}

impl CPU {
    /// new returns a new CPU instance.
    fn new() -> CPU {
        CPU {
            registers: [0; REGISTER_SIZE],
            memory: [0; MEMORY_SIZE],
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
            program_counter: 0,
        }
    }

    fn run(&mut self) {
        loop {
            let op_hi = self.memory[self.program_counter] as u16;
            let op_lo = self.memory[self.program_counter + 1] as u16;
            let opcode = op_hi << 8 | op_lo;

            let x = ((opcode & 0x0F00) >> 8) as u8;
            let y = ((opcode & 0x00F0) >> 4) as u8;
            let kk = (opcode & 0x00FF) as u8;
            let op_minor = (opcode & 0x000F) as u8;
            let nnn = opcode & 0x0FFF;

            self.program_counter += 2;

            match opcode {
                0x0000 => {
                    return;
                }
                0x00E0 => self.cls(),
                0x00EE => self.ret(),
                0x1000..=0x1FFF => self.jmp(nnn),
                0x2000..=0x2FFF => self.call(nnn),
                0x3000..=0x3FFF => self.se(x, kk),
                0x4000..=0x4FFF => self.sne(x, kk),
                0x5000..=0x5FFF => self.se(x, y),
                0x6000..=0x6FFF => self.ld(x, kk),
                0x7000..=0x7FFF => self.add(x, kk),
                0x800..=0x8FFF => match op_minor {
                    0x0 => self.ld(x, self.registers[y as usize]),
                    0x1 => self.or_xy(x, y),
                    0x2 => self.and_xy(x, y),
                    0x3 => self.xor_xy(x, y),
                    0x4 => self.add_xy(x, y),
                    _ => panic!("Unknown opcode: {:X}", opcode),
                },
                _ => panic!("Unknown opcode: {:X}", opcode),
            }
        }
    }

    /// (00E0) CLS clears the screen. (clear screen)
    fn cls(&mut self) {}

    /// (6xkk) LD sets 'kk' value into register 'vx'. (load)
    fn ld(&mut self, vx: u8, kk: u8) {
        self.registers[vx as usize] = kk;
    }

    /// (7xkk) ADD adds 'kk' value into register 'vx'. (add)
    fn add(&mut self, vx: u8, kk: u8) {
        self.registers[vx as usize] += kk;
    }

    /// SE skips the next instruction if register 'vx' equals 'kk'. (skip if equal)
    fn se(&mut self, vx: u8, kk: u8) {
        if vx == kk {
            self.program_counter += 2;
        }
    }

    /// SNE skips the next instruction if register 'vx' doesn't equal 'kk'. (skip if not equal)
    fn sne(&mut self, vx: u8, kk: u8) {
        if vx != kk {
            self.program_counter += 2;
        }
    }

    /// (1nnn) JMP jumps to address 'nnn'. (jump)
    fn jmp(&mut self, addr: u16) {
        self.program_counter = addr as usize;
    }

    /// (2nnn) CALL calls subroutine at 'nnn'. (call)
    fn call(&mut self, addr: u16) {
        if self.stack_pointer >= STACK_SIZE as u16 {
            panic!("Stack overflow");
        }
        self.stack[self.stack_pointer as usize] = self.program_counter as u16;
        self.stack_pointer += 1;
        self.program_counter = addr as usize;
    }

    /// (00ee) RET returns from a subroutine. (return)
    fn ret(&mut self) {
        if self.stack_pointer == 0 {
            panic!("Stack underflow");
        }
        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer as usize] as usize;
    }

    /// (7xkk) add xy adds register 'vx' and 'vy' and stores the result in 'vx'. (add)
    fn add_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] += self.registers[y as usize];

        // Set the carry flag.
        if self.registers[x as usize] < self.registers[y as usize] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn and_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] &= self.registers[y as usize];
    }

    fn or_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] |= self.registers[y as usize];
    }

    fn xor_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] ^= self.registers[y as usize];
    }
}

fn main() {
    let mut cpu = CPU::new();
    cpu.registers[0] = 5;
    cpu.registers[1] = 10;

    cpu.memory[0x000] = 0x21;
    cpu.memory[0x001] = 0x00;
    cpu.memory[0x002] = 0x21;
    cpu.memory[0x003] = 0x00;

    cpu.memory[0x100] = 0x80;
    cpu.memory[0x101] = 0x14;
    cpu.memory[0x102] = 0x80;
    cpu.memory[0x103] = 0x14;
    cpu.memory[0x104] = 0x00;
    cpu.memory[0x105] = 0xEE;

    cpu.run();

    assert_eq!(cpu.registers[0], 45);

    println!("5 + (10 * 2) + (10 * 2) = {}", cpu.registers[0]);
}
