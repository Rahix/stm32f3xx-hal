//! Serial Peripheral Interface (SPI) bus
//!
//! A usage example of the can peripheral can be found at [examples/spi.rs]
//!
//! [examples/spi.rs]: https://github.com/stm32-rs/stm32f3xx-hal/blob/v0.6.0/examples/spi.rs

use core::ptr;

use crate::hal::spi::FullDuplex;
pub use crate::hal::spi::{Mode, Phase, Polarity};
use crate::pac::{
    spi1::cr2::{DS_A, FRXTH_A},
    SPI1, SPI2, SPI3,
};
use crate::stm32::spi1;

#[cfg(any(
    feature = "stm32f302xd",
    feature = "stm32f302xe",
    feature = "stm32f303xd",
    feature = "stm32f303xe",
    feature = "stm32f398",
))]
use crate::stm32::SPI4;

#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
use crate::gpio::gpioa::{PA1, PA10, PA12, PA13, PA2, PA3, PA8, PA9};
#[cfg(any(
    feature = "stm32f302x6",
    feature = "stm32f302x8",
    feature = "stm32f302xd",
    feature = "stm32f302xe",
    feature = "stm32f303xd",
    feature = "stm32f303xe",
    feature = "stm32f318",
    feature = "stm32f398",
))]
use crate::gpio::gpioa::{PA10, PA11};
use crate::gpio::gpioa::{PA5, PA6, PA7};
#[cfg(any(
    feature = "stm32f301",
    feature = "stm32f302",
    feature = "stm32f303",
    feature = "stm32f318",
    feature = "stm32f334",
    feature = "stm32f358",
    feature = "stm32f398"
))]
use crate::gpio::gpiob::PB13;
#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
use crate::gpio::gpiob::{PB0, PB10, PB8};
use crate::gpio::gpiob::{PB14, PB15, PB5};
#[cfg(any(
    feature = "stm32f302",
    feature = "stm32f303",
    feature = "stm32f318",
    feature = "stm32f328",
    feature = "stm32f334",
    feature = "stm32f358",
    feature = "stm32f373",
    feature = "stm32f378",
    feature = "stm32f398",
))]
use crate::gpio::gpiob::{PB3, PB4};
use crate::gpio::gpioc::{PC10, PC11, PC12};
#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
use crate::gpio::gpioc::{PC2, PC3, PC7, PC8, PC9};
#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
use crate::gpio::gpiod::{PD3, PD4, PD7, PD8};
#[cfg(any(
    feature = "stm32f302xd",
    feature = "stm32f302xe",
    feature = "stm32f303xd",
    feature = "stm32f303xe",
    feature = "stm32f398",
))]
use crate::gpio::gpioe::{PE12, PE13, PE14, PE2, PE5, PE6};
#[cfg(any(
    feature = "stm32f302x6",
    feature = "stm32f302x8",
    feature = "stm32f302xd",
    feature = "stm32f302xe",
    feature = "stm32f303xd",
    feature = "stm32f303xe",
    feature = "stm32f318",
    feature = "stm32f398",
))]
use crate::gpio::gpiof::PF1;
#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
use crate::gpio::gpiof::PF6;
#[cfg(any(
    feature = "stm32f302xb",
    feature = "stm32f302xc",
    feature = "stm32f302xd",
    feature = "stm32f302xe",
    feature = "stm32f303xb",
    feature = "stm32f303xc",
    feature = "stm32f303xd",
    feature = "stm32f303xe",
    feature = "stm32f358",
    feature = "stm32f398",
))]
use crate::gpio::gpiof::{PF10, PF9};
use crate::gpio::{PushPull, AF5, AF6};
use crate::rcc::Clocks;
#[cfg(any(
    feature = "stm32f301",
    feature = "stm32f302",
    feature = "stm32f303xb",
    feature = "stm32f303xc",
    feature = "stm32f303xd",
    feature = "stm32f303xe",
    feature = "stm32f318",
    feature = "stm32f358",
    feature = "stm32f373",
    feature = "stm32f378",
    feature = "stm32f398"
))]
use crate::rcc::APB1;
#[cfg(any(
    feature = "stm32f302xb",
    feature = "stm32f302xc",
    feature = "stm32f302xd",
    feature = "stm32f302xe",
    feature = "stm32f303",
    feature = "stm32f328",
    feature = "stm32f334",
    feature = "stm32f358",
    feature = "stm32f373",
    feature = "stm32f378",
    feature = "stm32f398"
))]
use crate::rcc::APB2;
use crate::time::rate::*;
use core::marker::PhantomData;

/// SPI error
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Overrun occurred
    Overrun,
    /// Mode fault occurred
    ModeFault,
    /// CRC error
    Crc,
}

// FIXME these should be "closed" traits
/// SCK pin -- DO NOT IMPLEMENT THIS TRAIT
pub unsafe trait SckPin<SPI> {}

/// MISO pin -- DO NOT IMPLEMENT THIS TRAIT
pub unsafe trait MisoPin<SPI> {}

/// MOSI pin -- DO NOT IMPLEMENT THIS TRAIT
pub unsafe trait MosiPin<SPI> {}

unsafe impl SckPin<SPI1> for PA5<AF5<PushPull>> {}
#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl SckPin<SPI1> for PA12<AF6<PushPull>> {}
#[cfg(any(
    feature = "stm32f302xb",
    feature = "stm32f302xc",
    feature = "stm32f302xd",
    feature = "stm32f302xe",
    feature = "stm32f303",
    feature = "stm32f328",
    feature = "stm32f334",
    feature = "stm32f358",
    feature = "stm32f373",
    feature = "stm32f378",
    feature = "stm32f398",
))]
unsafe impl SckPin<SPI1> for PB3<AF5<PushPull>> {}
#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl SckPin<SPI1> for PC7<AF5<PushPull>> {}

#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl SckPin<SPI2> for PA8<AF5<PushPull>> {}
#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl SckPin<SPI2> for PB8<AF5<PushPull>> {}
#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl SckPin<SPI2> for PB10<AF5<PushPull>> {}
#[cfg(any(
    feature = "stm32f301",
    feature = "stm32f302",
    feature = "stm32f303",
    feature = "stm32f318",
    feature = "stm32f334",
    feature = "stm32f358",
    feature = "stm32f398"
))]
unsafe impl SckPin<SPI2> for PB13<AF5<PushPull>> {}
#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl SckPin<SPI2> for PD7<AF5<PushPull>> {}
#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl SckPin<SPI2> for PD8<AF5<PushPull>> {}
#[cfg(any(
    feature = "stm32f302x6",
    feature = "stm32f302x8",
    feature = "stm32f302xd",
    feature = "stm32f302xe",
    feature = "stm32f303xd",
    feature = "stm32f303xe",
    feature = "stm32f318",
    feature = "stm32f398",
))]
unsafe impl SckPin<SPI2> for PF1<AF5<PushPull>> {}
#[cfg(any(
    feature = "stm32f302xb",
    feature = "stm32f302xc",
    feature = "stm32f302xd",
    feature = "stm32f302xe",
    feature = "stm32f303xb",
    feature = "stm32f303xc",
    feature = "stm32f303xd",
    feature = "stm32f303xe",
    feature = "stm32f358",
    feature = "stm32f398",
))]
unsafe impl SckPin<SPI2> for PF9<AF5<PushPull>> {}
#[cfg(any(
    feature = "stm32f302xb",
    feature = "stm32f302xc",
    feature = "stm32f302xd",
    feature = "stm32f302xe",
    feature = "stm32f303xb",
    feature = "stm32f303xc",
    feature = "stm32f303xd",
    feature = "stm32f303xe",
    feature = "stm32f358",
    feature = "stm32f398",
))]
unsafe impl SckPin<SPI2> for PF10<AF5<PushPull>> {}

#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl SckPin<SPI3> for PA1<AF6<PushPull>> {}
#[cfg(any(
    feature = "stm32f302",
    feature = "stm32f303xd",
    feature = "stm32f303xe",
    feature = "stm32f318",
    feature = "stm32f373",
    feature = "stm32f378",
    feature = "stm32f398",
))]
unsafe impl SckPin<SPI3> for PB3<AF6<PushPull>> {}
unsafe impl SckPin<SPI3> for PC10<AF6<PushPull>> {}

#[cfg(any(
    feature = "stm32f302xd",
    feature = "stm32f302xe",
    feature = "stm32f303xd",
    feature = "stm32f303xe",
    feature = "stm32f398",
))]
unsafe impl SckPin<SPI4> for PE2<AF5<PushPull>> {}
#[cfg(any(
    feature = "stm32f302xd",
    feature = "stm32f302xe",
    feature = "stm32f303xd",
    feature = "stm32f303xe",
    feature = "stm32f398",
))]
unsafe impl SckPin<SPI4> for PE12<AF5<PushPull>> {}

unsafe impl MisoPin<SPI1> for PA6<AF5<PushPull>> {}
#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl MisoPin<SPI1> for PA13<AF6<PushPull>> {}
#[cfg(any(
    feature = "stm32f302xb",
    feature = "stm32f302xc",
    feature = "stm32f302xd",
    feature = "stm32f302xe",
    feature = "stm32f303",
    feature = "stm32f328",
    feature = "stm32f334",
    feature = "stm32f358",
    feature = "stm32f373",
    feature = "stm32f378",
    feature = "stm32f398",
))]
unsafe impl MisoPin<SPI1> for PB4<AF5<PushPull>> {}
#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl MisoPin<SPI1> for PC8<AF5<PushPull>> {}

#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl MisoPin<SPI2> for PA9<AF5<PushPull>> {}
#[cfg(any(
    feature = "stm32f302x6",
    feature = "stm32f302x8",
    feature = "stm32f302xd",
    feature = "stm32f302xe",
    feature = "stm32f303xd",
    feature = "stm32f303xe",
    feature = "stm32f318",
    feature = "stm32f398",
))]
unsafe impl MisoPin<SPI2> for PA10<AF5<PushPull>> {}
unsafe impl MisoPin<SPI2> for PB14<AF5<PushPull>> {}
#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl MisoPin<SPI2> for PC2<AF5<PushPull>> {}
#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl MisoPin<SPI2> for PD3<AF5<PushPull>> {}

#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl MisoPin<SPI3> for PA2<AF6<PushPull>> {}
#[cfg(any(
    feature = "stm32f302",
    feature = "stm32f303xd",
    feature = "stm32f303xe",
    feature = "stm32f318",
    feature = "stm32f373",
    feature = "stm32f378",
    feature = "stm32f398",
))]
unsafe impl MisoPin<SPI3> for PB4<AF6<PushPull>> {}
unsafe impl MisoPin<SPI3> for PC11<AF6<PushPull>> {}

#[cfg(any(
    feature = "stm32f302xd",
    feature = "stm32f302xe",
    feature = "stm32f303xd",
    feature = "stm32f303xe",
    feature = "stm32f398",
))]
unsafe impl MisoPin<SPI4> for PE5<AF5<PushPull>> {}
#[cfg(any(
    feature = "stm32f302xd",
    feature = "stm32f302xe",
    feature = "stm32f303xd",
    feature = "stm32f303xe",
    feature = "stm32f398",
))]
unsafe impl MisoPin<SPI4> for PE13<AF5<PushPull>> {}

unsafe impl MosiPin<SPI1> for PA7<AF5<PushPull>> {}
#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl MosiPin<SPI1> for PB0<AF5<PushPull>> {}
unsafe impl MosiPin<SPI1> for PB5<AF5<PushPull>> {}
#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl MosiPin<SPI1> for PC9<AF5<PushPull>> {}
#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl MosiPin<SPI1> for PF6<AF5<PushPull>> {}

#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl MosiPin<SPI2> for PA10<AF5<PushPull>> {}
#[cfg(any(
    feature = "stm32f302x6",
    feature = "stm32f302x8",
    feature = "stm32f302xd",
    feature = "stm32f302xe",
    feature = "stm32f303xd",
    feature = "stm32f303xe",
    feature = "stm32f318",
    feature = "stm32f398",
))]
unsafe impl MosiPin<SPI2> for PA11<AF5<PushPull>> {}
unsafe impl MosiPin<SPI2> for PB15<AF5<PushPull>> {}
#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl MisoPin<SPI2> for PC3<AF5<PushPull>> {}
#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl MisoPin<SPI2> for PD4<AF5<PushPull>> {}

#[cfg(any(feature = "stm32f373", feature = "stm32f378"))]
unsafe impl MisoPin<SPI3> for PA3<AF6<PushPull>> {}
unsafe impl MosiPin<SPI3> for PB5<AF6<PushPull>> {}
unsafe impl MosiPin<SPI3> for PC12<AF6<PushPull>> {}

#[cfg(any(
    feature = "stm32f302xd",
    feature = "stm32f302xe",
    feature = "stm32f303xd",
    feature = "stm32f303xe",
    feature = "stm32f398",
))]
unsafe impl MosiPin<SPI4> for PE6<AF5<PushPull>> {}
#[cfg(any(
    feature = "stm32f302xd",
    feature = "stm32f302xe",
    feature = "stm32f303xd",
    feature = "stm32f303xe",
    feature = "stm32f398",
))]
unsafe impl MosiPin<SPI4> for PE14<AF5<PushPull>> {}

/// Configuration trait for the Word Size
/// used by the SPI peripheral
pub trait Word {
    /// Returns the register configuration
    /// to set the word size
    fn register_config() -> (FRXTH_A, DS_A);
}

impl Word for u8 {
    fn register_config() -> (FRXTH_A, DS_A) {
        (FRXTH_A::QUARTER, DS_A::EIGHTBIT)
    }
}

impl Word for u16 {
    fn register_config() -> (FRXTH_A, DS_A) {
        (FRXTH_A::HALF, DS_A::SIXTEENBIT)
    }
}

/// SPI peripheral operating in full duplex master mode
pub struct Spi<SPI, PINS, WORD = u8> {
    spi: SPI,
    pins: PINS,
    _word: PhantomData<WORD>,
}

macro_rules! hal {
    ($($SPIX:ident: ($spiX:ident, $APBX:ident, $spiXen:ident, $spiXrst:ident, $pclkX:ident),)+) => {
        $(
            impl<SCK, MISO, MOSI, WORD> Spi<$SPIX, (SCK, MISO, MOSI), WORD> {
                /// Configures the SPI peripheral to operate in full duplex master mode
                pub fn $spiX(
                    spi: $SPIX,
                    pins: (SCK, MISO, MOSI),
                    mode: Mode,
                    freq: Hertz,
                    clocks: Clocks,
                    apb2: &mut $APBX,
                ) -> Self
                where
                    SCK: SckPin<$SPIX>,
                    MISO: MisoPin<$SPIX>,
                    MOSI: MosiPin<$SPIX>,
                    WORD: Word,
                {
                    // enable or reset $SPIX
                    apb2.enr().modify(|_, w| w.$spiXen().enabled());
                    apb2.rstr().modify(|_, w| w.$spiXrst().reset());
                    apb2.rstr().modify(|_, w| w.$spiXrst().clear_bit());

                    let (frxth, ds) = WORD::register_config();
                    spi.cr2.write(|w| {
                        w.frxth().variant(frxth);
                        w.ds().variant(ds);
                        // Slave Select output disabled
                        w.ssoe().disabled()
                    });

                    // CPHA: phase
                    // CPOL: polarity
                    // MSTR: master mode
                    // BR: 1 MHz
                    // SPE: SPI disabled
                    // LSBFIRST: MSB first
                    // SSM: enable software slave management (NSS pin free for other uses)
                    // SSI: set nss high = master mode
                    // CRCEN: hardware CRC calculation disabled
                    // BIDIMODE: 2 line unidirectional (full duplex)
                    spi.cr1.write(|w| {
                        w.mstr().master();

                        match mode.phase {
                            Phase::CaptureOnFirstTransition => w.cpha().first_edge(),
                            Phase::CaptureOnSecondTransition => w.cpha().second_edge(),
                        };

                        match mode.polarity {
                            Polarity::IdleLow => w.cpol().idle_low(),
                            Polarity::IdleHigh => w.cpol().idle_high(),
                        };

                        w.br().variant(Self::compute_baud_rate(clocks.$pclkX(), freq));

                        w.spe()
                            .enabled()
                            .lsbfirst()
                            .msbfirst()
                            .ssi()
                            .slave_not_selected()
                            .ssm()
                            .enabled()
                            .crcen()
                            .disabled()
                            .bidimode()
                            .unidirectional()
                    });

                    Spi { spi, pins, _word: PhantomData }
                }

                /// Releases the SPI peripheral and associated pins
                pub fn free(self) -> ($SPIX, (SCK, MISO, MOSI)) {
                    (self.spi, self.pins)
                }

                /// Change the baud rate of the SPI
                pub fn reclock(&mut self, freq: Hertz, clocks: Clocks) {
                    self.spi.cr1.modify(|_, w| w.spe().disabled());

                    self.spi.cr1.modify(|_, w| {
                        w.br().variant(Self::compute_baud_rate(clocks.$pclkX(), freq));
                        w.spe().enabled()
                    });
                }

                fn compute_baud_rate(clocks: Hertz, freq: Hertz) -> spi1::cr1::BR_A {
                    use spi1::cr1::BR_A;
                    match clocks.0 / *freq.integer() {
                        0 => crate::unreachable!(),
                        1..=2 => BR_A::DIV2,
                        3..=5 => BR_A::DIV4,
                        6..=11 => BR_A::DIV8,
                        12..=23 => BR_A::DIV16,
                        24..=39 => BR_A::DIV32,
                        40..=95 => BR_A::DIV64,
                        96..=191 => BR_A::DIV128,
                        _ => BR_A::DIV256,
                    }
                }


            }

            impl<PINS, WORD> FullDuplex<WORD> for Spi<$SPIX, PINS, WORD> {
                type Error = Error;

                fn read(&mut self) -> nb::Result<WORD, Error> {
                    let sr = self.spi.sr.read();

                    Err(if sr.ovr().is_overrun() {
                        nb::Error::Other(Error::Overrun)
                    } else if sr.modf().is_fault() {
                        nb::Error::Other(Error::ModeFault)
                    } else if sr.crcerr().is_no_match() {
                        nb::Error::Other(Error::Crc)
                    } else if sr.rxne().is_not_empty() {
                        let read_ptr = &self.spi.dr as *const _ as *const WORD;
                        // NOTE(unsafe) read from register owned by this Spi struct
                        let value = unsafe { ptr::read_volatile(read_ptr) };
                        return Ok(value);
                    } else {
                        nb::Error::WouldBlock
                    })
                }

                fn send(&mut self, word: WORD) -> nb::Result<(), Error> {
                    let sr = self.spi.sr.read();

                    Err(if sr.ovr().is_overrun() {
                        nb::Error::Other(Error::Overrun)
                    } else if sr.modf().is_fault() {
                        nb::Error::Other(Error::ModeFault)
                    } else if sr.crcerr().is_no_match() {
                        nb::Error::Other(Error::Crc)
                    } else if sr.txe().is_empty() {
                        let write_ptr = &self.spi.dr as *const _ as *mut WORD;
                        // NOTE(unsafe) write to register owned by this Spi struct
                        unsafe { ptr::write_volatile(write_ptr, word) };
                        return Ok(());
                    } else {
                        nb::Error::WouldBlock
                    })
                }
            }

            impl<PINS, WORD> crate::hal::blocking::spi::transfer::Default<WORD> for Spi<$SPIX, PINS, WORD> {}
            impl<PINS, WORD> crate::hal::blocking::spi::write::Default<WORD> for Spi<$SPIX, PINS, WORD> {}
        )+
    }
}

#[cfg(any(
    feature = "stm32f303x6",
    feature = "stm32f303x8",
    feature = "stm32f328",
    feature = "stm32f334",
))]
hal! {
    SPI1: (spi1, APB2, spi1en, spi1rst, pclk2),
}

#[cfg(any(
    feature = "stm32f301",
    feature = "stm32f302x6",
    feature = "stm32f302x8",
    feature = "stm32f318",
))]
hal! {
    SPI2: (spi2, APB1, spi2en, spi2rst, pclk1),
    SPI3: (spi3, APB1, spi3en, spi3rst, pclk1),
}

#[cfg(any(
    feature = "stm32f302xb",
    feature = "stm32f302xc",
    feature = "stm32f303xb",
    feature = "stm32f303xc",
    feature = "stm32f358",
    feature = "stm32f373",
    feature = "stm32f378",
))]
hal! {
    SPI1: (spi1, APB2, spi1en, spi1rst, pclk2),
    SPI2: (spi2, APB1, spi2en, spi2rst, pclk1),
    SPI3: (spi3, APB1, spi3en, spi3rst, pclk1),
}

#[cfg(any(
    feature = "stm32f302xd",
    feature = "stm32f302xe",
    feature = "stm32f303xd",
    feature = "stm32f303xe",
    feature = "stm32f398",
))]
hal! {
    SPI1: (spi1, APB2, spi1en, spi1rst, pclk2),
    SPI2: (spi2, APB1, spi2en, spi2rst, pclk1),
    SPI3: (spi3, APB1, spi3en, spi3rst, pclk1),
    SPI4: (spi4, APB2, spi4en, spi4rst, pclk2),
}
