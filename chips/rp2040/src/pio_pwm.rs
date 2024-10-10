use crate::clocks;
use crate::gpio::RPGpio;
use crate::pio::{PIONumber, Pio, SMNumber, StateMachineConfiguration};

use kernel::utilities::cells::OptionalCell;
use kernel::{hil, ErrorCode};

pub struct PioPwm<'a> {
    clocks: OptionalCell<&'a clocks::Clocks>,
}

impl<'a> PioPwm<'a> {
    pub fn start_pwm() {}
}

impl<'a> hil::pwm::Pwm for PioPwm<'a> {
    type Pin = RPGpio;

    fn start(
        &self,
        pin: &Self::Pin,
        frequency_hz: usize,
        duty_cycle_percentage: usize,
    ) -> Result<(), ErrorCode> {
        let mut pio: Pio = Pio::new_pio0();

        // Ramps up the intensity of an LED using PWM.
        // .program pwm
        // .side_set 1 opt
        //     pull noblock    side 0 ; Pull from FIFO to OSR if available, else copy X to OSR.
        //     mov x, osr             ; Copy most-recently-pulled value back to scratch X
        //     mov y, isr             ; ISR contains PWM period. Y used as counter.
        // countloop:
        //     jmp x!=y noset         ; Set pin high if X == Y, keep the two paths length matched
        //     jmp skip        side 1
        // noset:
        //     nop                    ; Single dummy cycle to keep the two paths the same length
        // skip:
        //     jmp y-- countloop      ; Loop until Y hits 0, then pull a fresh PWM value from FIFO
        let path: [u8; 14] = [
            0x90, 0x80, 0xa0, 0x27, 0xa0, 0x46, 0x00, 0xa5, 0x18, 0x06, 0xa0, 0x42, 0x00, 0x83,
        ];

        pio.init();
        pio.add_program(&path);
        let mut custom_config = StateMachineConfiguration::default();

        let pin_nr = pin as *const _ as u32;
        custom_config.side_set_base = pin_nr;
        custom_config.side_set_bit_count = 2;
        custom_config.side_set_opt_enable = true;
        custom_config.side_set_pindirs = false;
        let max_freq = self.get_maximum_frequency_hz();
        let pwm_period = (max_freq / frequency_hz) as u32;
        let sm_number = SMNumber::SM0;
        let duty_cycle = (duty_cycle_percentage / 100) as u32;
        pio.pwm_program_init(
            PIONumber::PIO0,
            sm_number,
            pin_nr,
            pwm_period,
            &custom_config,
        );
        pio.sm_put_blocking(sm_number, pwm_period * duty_cycle / 100);
        Ok(())
    }

    fn stop(&self, _pin: &Self::Pin) -> Result<(), ErrorCode> {
        let pio: Pio = Pio::new_pio0();
        // pio.sm_put_blocking(SMNumber::SM0, 0);
        pio.clear_instr_registers();
        Ok(())
    }

    fn get_maximum_duty_cycle(&self) -> usize {
        // u16::MAX as usize + 1
        // being a percentage, max duty cycle is 100
        100
    }

    fn get_maximum_frequency_hz(&self) -> usize {
        self.clocks
            .unwrap_or_panic()
            .get_frequency(clocks::Clock::System) as usize
    }
}
