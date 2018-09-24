//! Chip and board specific configuration settings go here.
use bootload;
use stm32f7x7;

/// TCP port to listen on
pub const TCP_PORT: u16 = 7777;

/// PHY address
pub const ETH_PHY_ADDR: u8 = 0;

/// Start address of each sector in flash
pub const FLASH_SECTOR_ADDRESSES: [u32; 12] = [
    0x0800_0000,
    0x0800_8000,
    0x0801_0000,
    0x0801_8000,
    0x0802_0000,
    0x0804_0000,
    0x0808_0000,
    0x080C_0000,
    0x0810_0000,
    0x0814_0000,
    0x0818_0000,
    0x081C_0000,
];

/// Final valid address in flash
pub const FLASH_END: u32 = 0x081F_FFFF;

/// Address of configuration sector. Must be one of the start addresses in
/// FLASH_SECTOR_ADDRESSES.
pub const FLASH_CONFIG: u32 = FLASH_SECTOR_ADDRESSES[3];

/// Address of user firmware sector. Must be one of the start addresses in
/// FLASH_SECTOR_ADDRESSES.
pub const FLASH_USER: u32 = FLASH_SECTOR_ADDRESSES[4];

/// Magic value used in this module to check if bootloader should start.
pub const BOOTLOAD_FLAG_VALUE: u32 = 0xB00110AD;
/// Address of magic value used in this module to check if bootloader should
/// start.
/// SRAM1
//pub const BOOTLOAD_FLAG_ADDRESS: u32 = 0x2002_0000;
// DTCM RAM
pub const BOOTLOAD_FLAG_ADDRESS: u32 = 0x2000_0000;

/// This function should return true if the bootloader should enter bootload
/// mode, or false to immediately chainload the user firmware.
///
/// By default we check if there was a software reset and a magic value is set
/// in RAM, but you could also check GPIOs etc here.
///
/// Ensure any state change to the peripherals is reset before returning from
/// this function.
pub fn should_enter_bootloader(peripherals: &mut stm32f7x7::Peripherals) -> bool {
    // Our plan is:
    // * If the reset was a software reset, and the magic flag is in the magic
    // location,   then the user firmware requested bootload, so enter bootload.
    // * Otherwise we check if PD2 is LOW for at least a full byte period of the
    // UART,   indicating someone has connected 3V to the external connector.
    let cond1 = bootload::was_software_reset(&mut peripherals.RCC) && bootload::flag_set();

    peripherals.RCC.ahb1enr.modify(|_, w| w.gpioden().enabled());
    peripherals.GPIOD.moder.modify(|_, w| w.moder2().input());

    let hsi_clk = 16_000_000;
    let sync_baud = 1_000_000;
    let bit_periods = 10;
    let delay = (hsi_clk / sync_baud) * bit_periods;
    let mut cond2 = true;
    for _ in 0..delay {
        cond2 &= peripherals.GPIOD.idr.read().idr2().bit_is_clear();
    }

    peripherals
        .RCC
        .ahb1enr
        .modify(|_, w| w.gpioden().disabled());
    cond1 || cond2
}

/// Set up GPIOs for ethernet.
///
/// You should enable 9 GPIOs used by the ethernet controller. All GPIO clocks
/// are already enabled. This is also a sensible place to turn on an LED or
/// similar to indicate bootloader mode.
pub fn configure_gpio(peripherals: &mut stm32f7x7::Peripherals) {
    let gpioa = &peripherals.GPIOA;
    let gpiob = &peripherals.GPIOB;
    let gpioc = &peripherals.GPIOC;
    let gpioe = &peripherals.GPIOE;
    let gpiog = &peripherals.GPIOG;

    // Status LED
    gpioe.moder.modify(|_, w| w.moder7().output());
    gpioe.odr.modify(|_, w| w.odr7().clear_bit());

    // Configure ethernet related GPIO:
    // GPIOA 1, 2, 7
    // GPIOB 13 (why 11 and 12?)
    // GPIOC 1, 4, 5
    // GPIOG 2, 11, 13
    // All set to AF11 and very high speed.
    gpioa.moder.modify(|_, w| {
        w.moder1()
            .alternate()
            .moder2()
            .alternate()
            .moder7()
            .alternate()
    });
    gpiob.moder.modify(|_, w| {
        w.moder11()
            .alternate()
            .moder12()
            .alternate()
            .moder13()
            .alternate()
    });
    gpioc.moder.modify(|_, w| {
        w.moder1()
            .alternate()
            .moder4()
            .alternate()
            .moder5()
            .alternate()
    });
    gpiog.moder.modify(|_, w| {
        w.moder2()
            .alternate()
            .moder11()
            .alternate()
            .moder13()
            .alternate()
    });
    gpioa.ospeedr.modify(|_, w| {
        w.ospeedr1()
            .very_high_speed()
            .ospeedr2()
            .very_high_speed()
            .ospeedr7()
            .very_high_speed()
    });
    gpiob.ospeedr.modify(|_, w| {
        w.ospeedr11()
            .very_high_speed()
            .ospeedr12()
            .very_high_speed()
            .ospeedr13()
            .very_high_speed()
    });
    gpioc.ospeedr.modify(|_, w| {
        w.ospeedr1()
            .very_high_speed()
            .ospeedr4()
            .very_high_speed()
            .ospeedr5()
            .very_high_speed()
    });
    gpiog.ospeedr.modify(|_, w| {
        w.ospeedr2()
            .very_high_speed()
            .ospeedr11()
            .very_high_speed()
            .ospeedr13()
            .very_high_speed()
    });
    gpioa
        .afrl
        .modify(|_, w| w.afrl1().af11().afrl2().af11().afrl7().af11());
    gpiob
        .afrh
        .modify(|_, w| w.afrh11().af11().afrh12().af11().afrh13().af11());
    gpioc
        .afrl
        .modify(|_, w| w.afrl1().af11().afrl4().af11().afrl5().af11());
    gpiog.afrl.modify(|_, w| w.afrl2().af11());
    gpiog.afrh.modify(|_, w| w.afrh11().af11().afrh13().af11());
}
