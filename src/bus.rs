#[derive(Clone, Copy)]
pub enum BusAction {
    PpuAction(PpuAction),
    None
}

#[derive(Clone, Copy)]
pub enum PpuAction {
    PpuCtrlWrite(u8),
    PpuMaskWrite(u8),
    PpuStatusRead,
    OmaAddrWrite(u8),
    OamDataWrite(u8),
    PpuScrollWrite(u8),
    PpuAddrWrite(u8),
    PpuDataRead,
    PpuDataWrite(u8),
    OamDmaWrite(u8),
    None,
}
