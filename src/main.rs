use chip8::cpu::CPU;

fn main() {
    let mut cpu = CPU::new("test_opcode.ch8");
    cpu.run()
}
