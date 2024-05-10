mod mapper0;

pub use self::mapper0::Mapper0;

pub trait Mapper {
    fn mapped_cpu_read(&mut self, addr: usize) -> usize;

    fn mapped_cpu_write(&mut self, addr: usize, byte: u8) -> usize;

    fn mapped_ppu_read(&mut self, addr: usize) -> usize;

    fn mapped_ppu_write(&mut self, addr: usize, byte: u8) -> usize;
}