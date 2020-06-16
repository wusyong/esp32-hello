use bitflags::bitflags;

use crate::{ptr_set_mask, ptr_clear_mask};

const DR_REG_DPORT_BASE: u32 = 0x3ff00000;

const DPORT_PERIP_CLK_EN_REG: u32 = DR_REG_DPORT_BASE + 0x0C0;
const DPORT_PERIP_RST_EN_REG: u32 = DR_REG_DPORT_BASE + 0x0C4;

const DPORT_UART_CLK_EN:  u32 = 1 << 2;
const DPORT_UART1_CLK_EN: u32 = 1 << 5;
const DPORT_UART2_CLK_EN: u32 = 1 << 23;

const DPORT_UART_RST: u32 = 1 << 2;
const DPORT_UART1_RST: u32 = 1 << 5;
const DPORT_UART2_RST: u32 = 1 << 23;

bitflags! {
  #[derive(Default)]
  pub struct FlowControl: u8 {
    const DISABLED = 0b000;
    const RTS      = 0b001;
    const CTS      = 0b010;
    const CTS_RTS  = Self::CTS.bits | Self::RTS.bits;
  }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct uart_dev_t {
  fifo: u32,
  int_raw: u32,
  int_st: u32,
  int_ena: u32,
  int_clr: u32,
  clk_div: u32,
  auto_baud: u32,
  status: u32,
  conf0: u32,
  conf1: u32,
  lowpulse: u32,
  highpulse: u32,
  rxd_cnt: u32,
  flow_conf: u32,
  sleep_conf: u32,
  swfc_conf: u32,
  idle_conf: u32,
  rs485_conf: u32,
  at_cmd_precnt: u32,
  at_cmd_postcnt: u32,
  at_cmd_gaptout: u32,
  at_cmd_char: u32,
  mem_conf: u32,
  mem_tx_status: u32,
  mem_rx_status: u32,
  mem_cnt_status: u32,
  pospulse: u32,
  negpulse: u32,
  reserved_70: u32,
  reserved_74: u32,
  date: u32,
  id: u32,
}

// extern "C" {
//   static mut UART0: uart_dev_t;
//   static mut UART1: uart_dev_t;
//   static mut UART2: uart_dev_t;
// }

static mut UART: [Option<u32>; 3] = [
  Some(0x3ff40000),
  Some(0x3ff50000),
  Some(0x3ff6e000),
];

fn get(i: usize) -> Option<uart_dev_t> {
  unsafe { UART[i].take().map(|address| *(address as *mut uart_dev_t) ) }
}

macro_rules! uart {
  ($uart:ident, $num:expr, $clock_bit:expr, $reset_bit:expr) => {
    pub struct $uart;

    impl $uart {
      const CLOCK_BIT: u32 = $clock_bit;
      const RESET_BIT: u32 = $reset_bit;

      pub fn new() -> Self {
        unsafe {
          ptr_set_mask!(DPORT_PERIP_CLK_EN_REG, Self::CLOCK_BIT);
          ptr_clear_mask!(DPORT_PERIP_RST_EN_REG, Self::RESET_BIT);
        }

        let mut uart = Self;

        uart.set_flow_control(FlowControl::RTS);

        uart
      }

      fn set_flow_control(&mut self, flow_control: FlowControl) {
        if flow_control.contains(FlowControl::RTS) {

        } else {

        }

      }
    }

    impl Drop for $uart {
      fn drop(&mut self) {
        unsafe {
          ptr_clear_mask!(DPORT_PERIP_CLK_EN_REG, Self::CLOCK_BIT);
          ptr_set_mask!(DPORT_PERIP_RST_EN_REG, Self::RESET_BIT);
        }
      }
    }
  }
}

uart!(UART0, 0, DPORT_UART_CLK_EN,  DPORT_UART_RST);
uart!(UART1, 1, DPORT_UART1_CLK_EN, DPORT_UART1_RST);
uart!(UART2, 2, DPORT_UART2_CLK_EN, DPORT_UART2_RST);
