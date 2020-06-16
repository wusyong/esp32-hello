use core::ptr;

use bitflags::bitflags;

use crate::{ptr_set_mask, ptr_clear_mask};

const DR_REG_IO_MUX_BASE: usize = 0x3ff49000;
const IO_MUX_GPIO0_REG: usize = DR_REG_IO_MUX_BASE + 0x44;
const IO_MUX_GPIO1_REG: usize = DR_REG_IO_MUX_BASE + 0x88;
const IO_MUX_GPIO2_REG: usize = DR_REG_IO_MUX_BASE + 0x40;
const IO_MUX_GPIO3_REG: usize = DR_REG_IO_MUX_BASE + 0x84;
const IO_MUX_GPIO4_REG: usize = DR_REG_IO_MUX_BASE + 0x48;
const IO_MUX_GPIO5_REG: usize = DR_REG_IO_MUX_BASE + 0x6c;
const IO_MUX_GPIO6_REG: usize = DR_REG_IO_MUX_BASE + 0x60;
const IO_MUX_GPIO7_REG: usize = DR_REG_IO_MUX_BASE + 0x64;
const IO_MUX_GPIO8_REG: usize = DR_REG_IO_MUX_BASE + 0x68;
const IO_MUX_GPIO9_REG: usize = DR_REG_IO_MUX_BASE + 0x54;
const IO_MUX_GPIO10_REG: usize = DR_REG_IO_MUX_BASE + 0x58;
const IO_MUX_GPIO11_REG: usize = DR_REG_IO_MUX_BASE + 0x5c;
const IO_MUX_GPIO12_REG: usize = DR_REG_IO_MUX_BASE + 0x34;
const IO_MUX_GPIO13_REG: usize = DR_REG_IO_MUX_BASE + 0x38;
const IO_MUX_GPIO14_REG: usize = DR_REG_IO_MUX_BASE + 0x30;
const IO_MUX_GPIO15_REG: usize = DR_REG_IO_MUX_BASE + 0x3c;
const IO_MUX_GPIO16_REG: usize = DR_REG_IO_MUX_BASE + 0x4c;
const IO_MUX_GPIO17_REG: usize = DR_REG_IO_MUX_BASE + 0x50;
const IO_MUX_GPIO18_REG: usize = DR_REG_IO_MUX_BASE + 0x70;
const IO_MUX_GPIO19_REG: usize = DR_REG_IO_MUX_BASE + 0x74;
// const IO_MUX_GPIO20_REG: usize = DR_REG_IO_MUX_BASE + 0x78;
const IO_MUX_GPIO21_REG: usize = DR_REG_IO_MUX_BASE + 0x7c;
const IO_MUX_GPIO22_REG: usize = DR_REG_IO_MUX_BASE + 0x80;
const IO_MUX_GPIO23_REG: usize = DR_REG_IO_MUX_BASE + 0x8c;
// const IO_MUX_GPIO24_REG: usize = DR_REG_IO_MUX_BASE + 0x90;
const IO_MUX_GPIO25_REG: usize = DR_REG_IO_MUX_BASE + 0x24;
const IO_MUX_GPIO26_REG: usize = DR_REG_IO_MUX_BASE + 0x28;
const IO_MUX_GPIO27_REG: usize = DR_REG_IO_MUX_BASE + 0x2c;
const IO_MUX_GPIO32_REG: usize = DR_REG_IO_MUX_BASE + 0x1c;
const IO_MUX_GPIO33_REG: usize = DR_REG_IO_MUX_BASE + 0x20;
const IO_MUX_GPIO34_REG: usize = DR_REG_IO_MUX_BASE + 0x14;
const IO_MUX_GPIO35_REG: usize = DR_REG_IO_MUX_BASE + 0x18;
const IO_MUX_GPIO36_REG: usize = DR_REG_IO_MUX_BASE + 0x04;
const IO_MUX_GPIO37_REG: usize = DR_REG_IO_MUX_BASE + 0x08;
const IO_MUX_GPIO38_REG: usize = DR_REG_IO_MUX_BASE + 0x0c;
const IO_MUX_GPIO39_REG: usize = DR_REG_IO_MUX_BASE + 0x10;

const DR_REG_GPIO_BASE: usize = 0x3ff44000;
const GPIO_FUNC0_OUT_SEL_CFG_REG: usize = DR_REG_GPIO_BASE + 0x0530;

const GPIO_PIN_COUNT: usize = 40;

const GPIO_PIN_MUX_REG: [usize; GPIO_PIN_COUNT] = [
  IO_MUX_GPIO0_REG,
  IO_MUX_GPIO1_REG,
  IO_MUX_GPIO2_REG,
  IO_MUX_GPIO3_REG,
  IO_MUX_GPIO4_REG,
  IO_MUX_GPIO5_REG,
  IO_MUX_GPIO6_REG,
  IO_MUX_GPIO7_REG,
  IO_MUX_GPIO8_REG,
  IO_MUX_GPIO9_REG,
  IO_MUX_GPIO10_REG,
  IO_MUX_GPIO11_REG,
  IO_MUX_GPIO12_REG,
  IO_MUX_GPIO13_REG,
  IO_MUX_GPIO14_REG,
  IO_MUX_GPIO15_REG,
  IO_MUX_GPIO16_REG,
  IO_MUX_GPIO17_REG,
  IO_MUX_GPIO18_REG,
  IO_MUX_GPIO19_REG,
  0,
  IO_MUX_GPIO21_REG,
  IO_MUX_GPIO22_REG,
  IO_MUX_GPIO23_REG,
  0,
  IO_MUX_GPIO25_REG,
  IO_MUX_GPIO26_REG,
  IO_MUX_GPIO27_REG,
  0,
  0,
  0,
  0,
  IO_MUX_GPIO32_REG,
  IO_MUX_GPIO33_REG,
  IO_MUX_GPIO34_REG,
  IO_MUX_GPIO35_REG,
  IO_MUX_GPIO36_REG,
  IO_MUX_GPIO37_REG,
  IO_MUX_GPIO38_REG,
  IO_MUX_GPIO39_REG,
];

bitflags! {
  #[derive(Default)]
  pub struct GpioMode: u8 {
    const DISABLED                = 0b000;
    const INPUT                   = 0b001;
    const OUTPUT                  = 0b010;
    const OPEN_DRAIN              = 0b100;
    const OUTPUT_OPEN_DRAIN       = Self::OUTPUT.bits | Self::OPEN_DRAIN.bits;
    const INPUT_OUTPUT_OPEN_DRAIN = Self::INPUT.bits | Self::OUTPUT.bits | Self::OPEN_DRAIN.bits;
    const INPUT_OUTPUT            = Self::INPUT.bits | Self::OUTPUT.bits;
  }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct gpio_dev_t {
  pub bt_select: u32,
  pub out: u32,
  pub out_w1ts: u32,
  pub out_w1tc: u32,
  pub out1: u32,
  pub out1_w1ts: u32,
  pub out1_w1tc: u32,
  pub sdio_select: u32,
  pub enable: u32,
  pub enable_w1ts: u32,
  pub enable_w1tc: u32,
  pub enable1: u32,
  pub enable1_w1ts: u32,
  pub enable1_w1tc: u32,
  pub strap: u32,
  pub r#in: u32,
  pub in1: u32,
  pub status: u32,
  pub status_w1ts: u32,
  pub status_w1tc: u32,
  pub status1: u32,
  pub status1_w1ts: u32,
  pub status1_w1tc: u32,
  pub reserved_5c: u32,
  pub acpu_int: u32,
  pub acpu_nmi_int: u32,
  pub pcpu_int: u32,
  pub pcpu_nmi_int: u32,
  pub cpusdio_int: u32,
  pub acpu_int1: u32,
  pub acpu_nmi_int1: u32,
  pub pcpu_int1: u32,
  pub pcpu_nmi_int1: u32,
  pub cpusdio_int1: u32,
  pub pin: [u32; 40],
  pub cali_conf: u32,
  pub cali_data: u32,
  pub func_in_sel_cfg: [u32; 256],
  pub func_out_sel_cfg: [u32; 40],
}

extern "C" {
  pub fn gpio_pad_select_gpio(gpio_num: u32);
  fn gpio_matrix_out(gpio_num: u32, signal_idx: u32, out_inv: bool, oen_inv: bool);

  static mut GPIO: gpio_dev_t;
}

const INPUT_BIT: u32 = 0b1 << 9;
const PAD_DRIVER_BIT: u32 = 0b1 << 29;

unsafe fn gpio_set_direction(gpio_num: usize, mode: GpioMode) {
  let gpio_reg = GPIO_PIN_MUX_REG[gpio_num];
  if mode.contains(GpioMode::INPUT) {
    ptr_set_mask!(gpio_reg, INPUT_BIT);
  } else {
    ptr_clear_mask!(gpio_reg, INPUT_BIT);
  }

  if mode.contains(GpioMode::OUTPUT) {
    assert!(gpio_num < 34, "GPIO {} can only be an input.", gpio_num);

    if gpio_num < 32 {
      GPIO.enable_w1ts = 0b1 << gpio_num;
    } else {
      GPIO.enable1_w1ts = (0b1 << (gpio_num - 32)) << 24;
    }

    gpio_matrix_out(gpio_num as u32, SIG_GPIO_OUT_IDX, false, false);
  } else {
    if gpio_num < 32 {
      GPIO.enable_w1tc = 0b1 << gpio_num;
    } else {
      GPIO.enable1_w1tc = (0b1 << (gpio_num - 32)) << 24;
    }

    let ptr = (GPIO_FUNC0_OUT_SEL_CFG_REG + gpio_num * 4) as *mut u32;
    ptr::write_volatile(ptr, SIG_GPIO_OUT_IDX);
  }

  if mode.contains(GpioMode::OPEN_DRAIN) {
    GPIO.pin[gpio_num] |= PAD_DRIVER_BIT;
  } else {
    GPIO.pin[gpio_num] &= !PAD_DRIVER_BIT;
  }
}

const SIG_GPIO_OUT_IDX: u32 = 256;

pub fn gpio_set_level(gpio_num: usize, level: bool) {
  if level {
    gpio_set_high(gpio_num);
  } else {
    gpio_set_low(gpio_num);
  }
}

fn gpio_set_low(gpio_num: usize) {
  unsafe {
    if gpio_num < 32 {
      GPIO.out_w1tc = 0b1 << gpio_num;
    } else {
      GPIO.out1_w1tc = (0b1 << (gpio_num - 32)) << 24;
    }
  }
}

fn gpio_set_high(gpio_num: usize) {
  unsafe {
    if gpio_num < 32 {
      GPIO.out_w1ts = 0b1 << gpio_num;
    } else {
      GPIO.out1_w1ts = (0b1 << (gpio_num - 32)) << 24;
    }
  }
}

pub fn gpio_get_level(gpio_num: usize) -> bool {
  unsafe {
    if gpio_num < 32 {
      (GPIO.r#in >> gpio_num) & 0b1 == 1
    } else {
      ((GPIO.in1 >> 24) >> (gpio_num - 32)) & 0b1 == 1
    }
  }
}

use core::marker::PhantomData;


pub struct Output<MODE = ()> {
  _mode: PhantomData<MODE>,
}

pub struct OpenDrain;

pub struct Input<MODE = ()> {
  _mode: PhantomData<MODE>,
}

pub struct InputOutput<MODE = ()> {
  _mode: PhantomData<MODE>,
}

use embedded_hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin};

pub struct Pin<MODE> {
  i: u8,
  _mode: PhantomData<MODE>,
}

impl<T> Pin<T> {
  #[inline]
  unsafe fn new(gpio_num: u8, mode: GpioMode) -> Self {
    gpio_pad_select_gpio(gpio_num as u32);

    gpio_set_direction(gpio_num as usize, mode);

    Pin {
      i: gpio_num,
      _mode: PhantomData,
    }
  }
}

impl<MODE> OutputPin for Pin<Output<MODE>> {
  type Error = !;

  fn set_high(&mut self) -> Result<(), Self::Error> {
    gpio_set_high(self.i as usize);
    Ok(())
  }

  fn set_low(&mut self) -> Result<(), Self::Error> {
    gpio_set_low(self.i as usize);
    Ok(())
  }
}

impl<MODE> OutputPin for Pin<InputOutput<MODE>> {
  type Error = !;

  fn set_high(&mut self) -> Result<(), Self::Error> {
    gpio_set_high(self.i as usize);
    Ok(())
  }

  fn set_low(&mut self) -> Result<(), Self::Error> {
    gpio_set_low(self.i as usize);
    Ok(())
  }
}

impl<MODE> InputPin for Pin<InputOutput<MODE>> {
  type Error = !;

  fn is_high(&self) -> Result<bool, Self::Error> {
    Ok(gpio_get_level(self.i as usize) == true)
  }

  fn is_low(&self) -> Result<bool, Self::Error> {
    Ok(gpio_get_level(self.i as usize) == false)
  }
}

impl<MODE> InputPin for Pin<Input<MODE>> {
  type Error = !;

  fn is_high(&self) -> Result<bool, Self::Error> {
    Ok(gpio_get_level(self.i as usize) == true)
  }

  fn is_low(&self) -> Result<bool, Self::Error> {
    Ok(gpio_get_level(self.i as usize) == false)
  }
}

impl<MODE> StatefulOutputPin for Pin<InputOutput<MODE>> {
  fn is_set_high(&self) -> Result<bool, Self::Error> {
    Ok(gpio_get_level(self.i as usize) == true)
  }

  fn is_set_low(&self) -> Result<bool, Self::Error> {
    Ok(gpio_get_level(self.i as usize) == false)
  }
}

macro_rules! gpio {
  ($GPIOX:ident, $gpio_num:expr, $PXx:ident, $addr:expr) => {
    pub struct $GPIOX;

    impl $GPIOX {
      pub fn into_input() -> Pin<Input> {
        unsafe { Pin::new($gpio_num, GpioMode::INPUT) }
      }

      pub fn into_output() -> Pin<Output> {
        unsafe { Pin::new($gpio_num, GpioMode::OUTPUT) }
      }

      pub fn into_open_drain_output() -> Pin<Output<OpenDrain>> {
        unsafe { Pin::new($gpio_num, GpioMode::OUTPUT_OPEN_DRAIN) }
      }

      pub fn into_input_output() -> Pin<InputOutput> {
        unsafe { Pin::new($gpio_num, GpioMode::INPUT_OUTPUT) }
      }

      pub fn into_open_drain_input_output() -> Pin<InputOutput<OpenDrain>> {
        unsafe { Pin::new($gpio_num, GpioMode::INPUT_OUTPUT_OPEN_DRAIN) }
      }

      #[inline]
      pub fn register() -> *mut u32 {
        $addr as *mut u32
      }
    }
  }
}

gpio!(GPIO0,   0, P0, DR_REG_IO_MUX_BASE + 0x44);
gpio!(GPIO1,   1, P1, DR_REG_IO_MUX_BASE + 0x88);
gpio!(GPIO2,   2, P2, DR_REG_IO_MUX_BASE + 0x40);
gpio!(GPIO3,   3, P3, DR_REG_IO_MUX_BASE + 0x84);
gpio!(GPIO4,   4, P4, DR_REG_IO_MUX_BASE + 0x48);
gpio!(GPIO5,   5, P5, DR_REG_IO_MUX_BASE + 0x6c);
gpio!(GPIO6,   6, P6, DR_REG_IO_MUX_BASE + 0x60);
gpio!(GPIO7,   7, P7, DR_REG_IO_MUX_BASE + 0x64);
gpio!(GPIO8,   8, P8, DR_REG_IO_MUX_BASE + 0x68);
gpio!(GPIO9,   9, P9, DR_REG_IO_MUX_BASE + 0x54);
gpio!(GPIO10, 10, P10, DR_REG_IO_MUX_BASE + 0x58);
gpio!(GPIO11, 11, P11, DR_REG_IO_MUX_BASE + 0x5c);
gpio!(GPIO12, 12, P12, DR_REG_IO_MUX_BASE + 0x34);
gpio!(GPIO13, 13, P13, DR_REG_IO_MUX_BASE + 0x38);
gpio!(GPIO14, 14, P14, DR_REG_IO_MUX_BASE + 0x30);
gpio!(GPIO15, 15, P15, DR_REG_IO_MUX_BASE + 0x3c);
gpio!(GPIO16, 16, P16, DR_REG_IO_MUX_BASE + 0x4c);
gpio!(GPIO17, 17, P17, DR_REG_IO_MUX_BASE + 0x50);
gpio!(GPIO18, 18, P18, DR_REG_IO_MUX_BASE + 0x70);
gpio!(GPIO19, 19, P19, DR_REG_IO_MUX_BASE + 0x74);
gpio!(GPIO21, 21, P21, DR_REG_IO_MUX_BASE + 0x7c);
gpio!(GPIO22, 22, P22, DR_REG_IO_MUX_BASE + 0x80);
gpio!(GPIO23, 23, P23, DR_REG_IO_MUX_BASE + 0x8c);
gpio!(GPIO25, 25, P25, DR_REG_IO_MUX_BASE + 0x24);
gpio!(GPIO26, 26, P26, DR_REG_IO_MUX_BASE + 0x28);
gpio!(GPIO27, 27, P27, DR_REG_IO_MUX_BASE + 0x2c);
gpio!(GPIO32, 32, P32, DR_REG_IO_MUX_BASE + 0x1c);
gpio!(GPIO33, 33, P33, DR_REG_IO_MUX_BASE + 0x20);
gpio!(GPIO34, 34, P34, DR_REG_IO_MUX_BASE + 0x14);
gpio!(GPIO35, 35, P35, DR_REG_IO_MUX_BASE + 0x18);
gpio!(GPIO36, 36, P36, DR_REG_IO_MUX_BASE + 0x04);
gpio!(GPIO37, 37, P37, DR_REG_IO_MUX_BASE + 0x08);
gpio!(GPIO38, 38, P38, DR_REG_IO_MUX_BASE + 0x0c);
gpio!(GPIO39, 39, P39, DR_REG_IO_MUX_BASE + 0x10);
