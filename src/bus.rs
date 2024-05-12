use crate::cartridge::CartridgeNes;
use crate::ppu::PpuBus;

const CPU_RAM_START: usize = 0x0000;
const CPU_RAM_END: usize = 0x1FFF;
const PPU_REG_START: usize = 0x2000;
const PPU_REG_END: usize = 0x3FFF;
const IO_REG_START: usize = 0x4000;
const IO_REG_END: usize = 0x401F;
const CARTRIDGE_START: usize = 0x4020;
const CARTRIDGE_END: usize = 0xFFFF;

pub const DMA_REG_ADDR: usize = 0x4014;
const JOYPAD1_REG: usize = 0x4016;
const JOYPAD2_REG: usize = 0x4017;

const CPU_RAM_LENGTH: usize = 0x800;

pub struct Bus {
    cartridge: CartridgeNes,
    cpu_ram: [u8; CPU_RAM_LENGTH], 

    pub ppu_bus: PpuBus,

    joypad_registers: [u8; 2],
    joypad_state: [u8; 2],

    dma_page: u8,
    dma_addr: u8,
    dma_data: u8,
    pub dma_transferring: bool,
    false_dma: bool,

    // TODO: TEMPORARY
    io_registers: [u8; IO_REG_END - IO_REG_START + 1],
}

impl Bus {
    pub fn new(cartridge: CartridgeNes) -> Self {
        Self {
            cartridge,
            cpu_ram: [0; CPU_RAM_LENGTH],

            ppu_bus: PpuBus::new(),

            joypad_registers: [0; 2],
            joypad_state: [0; 2],

            dma_page: 0,
            dma_addr: 0,
            dma_data: 0,
            dma_transferring: false,
            false_dma: true,

            io_registers: [0; IO_REG_END - IO_REG_START + 1],
        }
    }

    pub fn cpu_read(&mut self, addr: usize) -> u8 {
        match addr {
            CPU_RAM_START..=CPU_RAM_END => self.cpu_ram[addr % CPU_RAM_LENGTH],
            PPU_REG_START..=PPU_REG_END => {
                self.ppu_bus.cpu_read_reg(addr, &mut self.cartridge)
            },
            DMA_REG_ADDR => 0,
            JOYPAD1_REG | JOYPAD2_REG => {
                let ret = (self.joypad_registers[addr & 0x01] & 0b10000000) != 0;
                self.joypad_registers[addr & 0x01] <<= 1;
                ret as u8
            }
            IO_REG_START..=IO_REG_END => self.io_registers[addr - IO_REG_START],
            CARTRIDGE_START..=CARTRIDGE_END => self.cartridge.cpu_read(addr),
            _ => unimplemented!()
        }
    }

    pub fn cpu_write(&mut self, addr: usize, byte: u8) {
        match addr {
            CPU_RAM_START..=CPU_RAM_END => self.cpu_ram[addr % CPU_RAM_LENGTH] = byte,
            PPU_REG_START..=PPU_REG_END => {
                self.ppu_bus.cpu_write_reg(addr, byte, &mut self.cartridge)
            },
            DMA_REG_ADDR => {
                self.dma_page = byte;
                self.dma_addr = 0x00;
                self.dma_transferring = true;
            }
            JOYPAD1_REG | JOYPAD2_REG => {
                self.joypad_registers[addr & 0x01] = self.joypad_state[addr & 0x01];
            },
            IO_REG_START..=IO_REG_END => self.io_registers[addr - IO_REG_START] = byte,
            CARTRIDGE_START..=CARTRIDGE_END => self.cartridge.cpu_write(addr, byte),
            _ => unimplemented!()
        }
    }

    pub fn dma_clock(&mut self, system_cycles: u32) {
        if self.false_dma {

            if system_cycles & 0x01 != 0  {
                self.false_dma = false;
            }
        } else {
            // read on even clock cycles, write on odd cycles
            if system_cycles & 0x01 == 0 {
                let data_addr = (self.dma_page as usize) << 8 | (self.dma_addr as usize);
                self.dma_data = self.cpu_read(data_addr);
            } else {
                self.ppu_bus.oam[self.dma_addr as usize] = self.dma_data;
                self.dma_addr = self.dma_addr.wrapping_add(1);

                if self.dma_addr == 0x00 {
                    self.dma_transferring = false;
                    self.false_dma = true;
                }
            }
        }
    }

    pub fn ppu_read(&mut self, addr: usize) -> u8 {
        self.ppu_bus.ppu_read(addr, &mut self.cartridge)
    }

    #[allow(dead_code)]
    pub fn ppu_write(&mut self, addr: usize, byte: u8) {
        self.ppu_bus.ppu_write(addr, byte, &mut self.cartridge);
    }

    // TODO: only one controller implemented
    pub fn update_joypad_state(&mut self, joypad_state: u8) {
        self.joypad_state[0] = joypad_state;
    }
}

#[cfg(test)]
impl Bus {
    pub fn load_ram(&mut self, data: &[u8]) {
        self.cpu_ram[..data.len()].copy_from_slice(&data[..data.len()]);
    }
}