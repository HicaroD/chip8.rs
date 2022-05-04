mod cpu;
mod display;
mod events;
mod memory;

use cpu::Cpu;
use display::Display;
use events::EventDriver;
use memory::Memory;

use sdl2::{event::Event, keyboard::Keycode, Sdl};

use std::{env, io, thread, time::Duration};

const SCALE_FACTOR: u32 = 20;
const WIDTH: u32 = 1280; // 64 * SCALE_FACTOR
const HEIGHT: u32 = 640; // 32 * SCALE_FACTOR

struct Chip8 {
    display: Display,
    cpu: Cpu,
    memory: Memory,
    event: EventDriver,
}

impl Chip8 {
    fn new(sdl_context: &Sdl) -> Self {
        Self {
            display: Display::new(sdl_context, WIDTH, HEIGHT),
            cpu: Cpu::new(),
            memory: Memory::new(),
            event: EventDriver::new(sdl_context),
        }
    }

    fn fetch_opcode(&self) -> u16 {
        (self.memory.ram[self.cpu.pc] as u16) << 8 | (self.memory.ram[self.cpu.pc + 1] as u16)
    }

    fn next_instruction(&mut self) {
        self.cpu.pc += 2;
    }

    fn skip_next_instruction_if(&mut self, condition: bool) {
        if condition {
            self.cpu.pc += 4;
        } else {
            self.cpu.pc += 2;
        }
    }

    fn execute_opcode(&mut self, opcode: u16) {
        let nibbles = (
            (opcode & 0xF000) >> 12,
            (opcode & 0x0F00) >> 8,
            (opcode & 0x00F0) >> 4,
            (opcode & 0x000F),
        );

        let x = nibbles.1; 
        let y = nibbles.2;
        let n = nibbles.3;
        let kk = opcode & 0x00FF;
        let nnn = opcode & 0x0FFF;

        match nibbles {

            // 0nnn - SYS addr
            (0x0, _, _, _) => {
                println!("OPCODE: 0nnn");
                self.next_instruction();
            }

            // 00E0 - CLS
            (0x0, 0x0, 0xE, 0x0) => {
                println!("OPCODE: 00E0");
                self.display.canvas.clear();
                self.next_instruction();
            }

            // 00EE - RET
            (0x0, 0x0, 0xE, 0xE) => {
                println!("OPCODE: 00EE");
                self.cpu.pc = self.memory.stack[self.cpu.sp] as usize;
                self.cpu.sp -= 1;
                self.next_instruction();
            }

            _ => {
                println!("Invalid opcode: {}", opcode);
            }
        }
    }

    fn execute_cycle(&mut self) {
        let mut opcode = self.fetch_opcode();
        println!("CURRENT OPCODE: {:#06X}", opcode);
        self.execute_opcode(opcode);
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Error: No input file");
        std::process::exit(1);
    }

    let sdl_context = sdl2::init().unwrap();
    let mut chip8 = Chip8::new(&sdl_context);

    let rom_path = &args[1];
    chip8.memory.load_rom(rom_path);
    chip8.memory.load_fontset();

    'running: loop {
        chip8.display.canvas.clear();
        let events = chip8.event.events.poll_iter();
        for event in events {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        chip8.execute_cycle();
        chip8.display.canvas.present();
    }
    Ok(())
}
