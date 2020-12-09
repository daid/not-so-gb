mod cpu;
mod video;

fn main() {
    let mut cpu = cpu::CPU::new("pokered.gbc");

    loop {
        cpu.step();
    }
}
