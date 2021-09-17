mod utils;
use utils::cpu_from_vec;

#[test]
fn lda_1() {
    let instr = vec![0xA9, 0x33];
    let mut cpu = cpu_from_vec(instr);
    for _ in 0..2 {
        cpu.tick();
    }
    assert_eq!(cpu.get_acc(), 0x33);
}
