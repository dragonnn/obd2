use defmt::info;

use super::{
    registers::{OperationMode, CANCTRL, CNF, RXB0CTRL, RXB1CTRL},
    AcceptanceFilter, IdHeader, CLKPRE,
};

/// Configuration for:
/// * Clock settings
/// * Operation Mode
/// * Receive buffers
/// * Receive buffer filters and masks
/// * Other flags inside the CANCTRL, CNF, RXB0CTRL, RXB1CTRL registers
#[derive(Clone, Debug, Default)]
pub struct Config<'a> {
    pub canctrl: CANCTRL,
    pub cnf: CNF,
    pub rxb0ctrl: RXB0CTRL,
    pub rxb1ctrl: RXB1CTRL,
    pub filters: &'a [(AcceptanceFilter, IdHeader)],
}

impl<'a> Config<'a> {
    #[inline]
    pub fn mode(mut self, mode: OperationMode) -> Self {
        self.canctrl.set_reqop(mode);
        self
    }
    pub fn set_clk_prescaler(mut self, clkpre: CLKPRE) -> Self {
        self.canctrl.set_clkpre(clkpre);
        self
    }
    #[inline]
    pub fn can_control_register(mut self, canctrl: CANCTRL) -> Self {
        self.canctrl = canctrl;
        self
    }
    #[inline]
    pub fn bitrate(mut self, cnf: CNF) -> Self {
        self.cnf = cnf;
        info!("self.cnf2.phseg1: {:?}", self.cnf.cnf2.phseg1());
        info!("self.cnf2.prseg: {:?}", self.cnf.cnf2.prseg());
        info!("self.cnf2.sam: {:?}", self.cnf.cnf2.sam());
        info!("self.cnf2.btlmode: {:?}", self.cnf.cnf2.btlmode());

        info!("self.cnf3.sof: {:?}", self.cnf.cnf3.sof());
        info!("self.cnf3.wakfil: {:?}", self.cnf.cnf3.wakfil());
        info!("self.cnf3.phseg2: {:?}", self.cnf.cnf3.phseg2());

        self.cnf.cnf2.set_sam(true);
        self.cnf.cnf2.set_btlmode(true);

        info!("self.canctrl.clken: {:?}", self.canctrl.clken());

        self
    }
    #[inline]
    pub fn receive_buffer_0(mut self, rxb0ctrl: RXB0CTRL) -> Self {
        self.rxb0ctrl = rxb0ctrl;
        self
    }
    #[inline]
    pub fn receive_buffer_1(mut self, rxb1ctrl: RXB1CTRL) -> Self {
        self.rxb1ctrl = rxb1ctrl;
        self
    }
    #[inline]
    pub fn filters(mut self, filters: &'a [(AcceptanceFilter, IdHeader)]) -> Self {
        self.filters = filters;
        self
    }
}
