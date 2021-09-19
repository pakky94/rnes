/// Combines two u8 values  
pub(crate) fn merge_u16(low: u8, high: u8) -> u16 {
    (high as u16) << 8 | (low as u16)
}

/// Splits the input in the lower and higher 8bit parts
pub(crate) fn split_u16(value: u16) -> (u8, u8) {
    let low = value as u8;
    let high = (value >> 8) as u8;
    (low, high)
}

pub(crate) fn overflowing_add_u8_i8(lhs: u8, rhs: i8) -> (u8, bool) {
    let lhs = lhs as u16;
    let rhs = rhs as u16;
    let res = lhs.wrapping_add(rhs);

    (res as u8, res >= 256)
}

#[derive(Debug, PartialEq)]
pub(crate) struct ADCSBCResult {
    pub(crate) result: u8,
    pub(crate) carry_flag: bool,
    pub(crate) overflow_flag: bool,
    pub(crate) zero_flag: bool,
    pub(crate) negative_flag: bool,
}
pub(crate) fn add_with_carry(lhs: u8, rhs: u8, carry_in: bool) -> ADCSBCResult {
    let (mut result, mut carry) = lhs.overflowing_add(rhs);
    //let mut overflow_flag = (lhs ^ result) & (rhs ^ result) & 0x80 != 0;

    if carry_in {
        let (acc2, carry2) = result.overflowing_add(1);
        //overflow_flag = overflow_flag | ((result ^ acc2) & (1 ^ acc2) & 0x80 != 0);
        result = acc2;
        carry = carry || carry2;
    }

    let carry_flag = carry;
    let zero_flag = result == 0;
    let negative_flag = result >= 128u8;

    let overflow_flag = {
        let lhs = lhs & 127;
        let rhs = rhs & 127;
        let mut res = lhs.wrapping_add(rhs);
        //let mut overflow_flag = (lhs ^ result) & (rhs ^ result) & 0x80 != 0;
        
        if carry_in {
            res = res.wrapping_add(1);
        }

        let c6 = res & 128 == 128;
        let c7 = carry_flag;

        c6 ^ c7
    };

    ADCSBCResult {
        result,
        carry_flag,
        overflow_flag,
        zero_flag,
        negative_flag,
    }
}

pub(crate) fn subtract_with_carry(lhs: u8, rhs: u8, carry_in: bool) -> ADCSBCResult {
    add_with_carry(lhs, !rhs, carry_in)
    //let (mut result, mut carry) = lhs.overflowing_sub(rhs);
    //let mut overflow_flag = (lhs ^ result) & (!rhs ^ result) & 0x80 != 0;
    //
    //if carry_in == false {
    //let (acc2, carry2) = result.overflowing_sub(1);
    //overflow_flag = overflow_flag | ((result ^ acc2) & (255 ^ acc2) & 0x80 != 0);
    //result = acc2;
    //carry = carry || carry2;
    //}
    //
    //let carry_flag = !carry;
    //let zero_flag = result == 0;
    //let negative_flag = result >= 128u8;
    //
    //ADCSBCResult {
    //result,
    //carry_flag,
    //overflow_flag,
    //zero_flag,
    //negative_flag,
    //}
}

#[derive(PartialEq)]
pub(crate) struct RotateResult {
    pub(crate) result: u8,
    pub(crate) carry_flag: bool,
    pub(crate) zero_flag: bool,
    pub(crate) negative_flag: bool,
}
impl std::fmt::Debug for RotateResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ result: {:#010b}, carry_flag: {}, zero_flag: {}, negative_flag: {} }}",
            self.result, self.carry_flag, self.zero_flag, self.negative_flag,
        )
    }
}
pub(crate) fn rotate_left(val: u8, carry_in: bool) -> RotateResult {
    let new_carry = val & 128u8 == 128;
    let mut res = val << 1;
    if carry_in {
        res = res | 1;
    }

    let carry_flag = new_carry;
    let zero_flag = res == 0;
    let negative_flag = res >= 128u8;

    RotateResult {
        result: res,
        carry_flag,
        zero_flag,
        negative_flag,
    }
}
pub(crate) fn rotate_right(val: u8, carry_in: bool) -> RotateResult {
    let new_carry = val & 1u8 == 1;
    let mut res = val >> 1;
    if carry_in {
        res = res | 128u8;
    }

    let carry_flag = new_carry;
    let zero_flag = res == 0;
    let negative_flag = res >= 128u8;

    RotateResult {
        result: res,
        carry_flag,
        zero_flag,
        negative_flag,
    }
}
pub(crate) fn shift_left(val: u8) -> RotateResult {
    let new_carry = val & 128u8 == 128;
    let res = val << 1;

    let carry_flag = new_carry;
    let zero_flag = res == 0;
    let negative_flag = res >= 128u8;

    RotateResult {
        result: res,
        carry_flag,
        zero_flag,
        negative_flag,
    }
}
pub(crate) fn shift_right(val: u8) -> RotateResult {
    let new_carry = (val & 1) == 1;
    let res = val >> 1;

    let carry_flag = new_carry;
    let zero_flag = res == 0;
    let negative_flag = res >= 128u8;

    RotateResult {
        result: res,
        carry_flag,
        zero_flag,
        negative_flag,
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct CompareU8Result {
    pub(crate) carry: bool,
    pub(crate) zero: bool,
    pub(crate) neg: bool,
}
pub(crate) fn compare_u8(lhs: u8, rhs: u8) -> CompareU8Result {
    let res = lhs.wrapping_sub(rhs);
    let carry = lhs >= rhs;
    let zero = res == 0;
    CompareU8Result {
        carry,
        zero,
        neg: res & 128 == 128,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_left_test() {
        let res = shift_left(0b11001010);
        assert_eq!(
            res,
            RotateResult {
                result: 0b10010100,
                carry_flag: true,
                zero_flag: false,
                negative_flag: true,
            }
        );

        let res = shift_left(0b01001010);
        assert_eq!(
            res,
            RotateResult {
                result: 0b10010100,
                carry_flag: false,
                zero_flag: false,
                negative_flag: true,
            }
        );

        let res = shift_left(0b10001010);
        assert_eq!(
            res,
            RotateResult {
                result: 0b00010100,
                carry_flag: true,
                zero_flag: false,
                negative_flag: false,
            }
        );

        let res = shift_left(0b00001010);
        assert_eq!(
            res,
            RotateResult {
                result: 0b00010100,
                carry_flag: false,
                zero_flag: false,
                negative_flag: false,
            }
        );
    }

    #[test]
    fn rotate_left_test() {
        let res = rotate_left(0b11001010, true);
        assert_eq!(
            res,
            RotateResult {
                result: 0b10010101,
                carry_flag: true,
                zero_flag: false,
                negative_flag: true,
            }
        );

        let res = rotate_left(0b11001010, false);
        assert_eq!(
            res,
            RotateResult {
                result: 0b10010100,
                carry_flag: true,
                zero_flag: false,
                negative_flag: true,
            }
        );

        let res = rotate_left(0b01001010, true);
        assert_eq!(
            res,
            RotateResult {
                result: 0b10010101,
                carry_flag: false,
                zero_flag: false,
                negative_flag: true,
            }
        );

        let res = rotate_left(0b01001010, false);
        assert_eq!(
            res,
            RotateResult {
                result: 0b10010100,
                carry_flag: false,
                zero_flag: false,
                negative_flag: true,
            }
        );

        let res = rotate_left(0b10001010, true);
        assert_eq!(
            res,
            RotateResult {
                result: 0b00010101,
                carry_flag: true,
                zero_flag: false,
                negative_flag: false,
            }
        );

        let res = rotate_left(0b00001010, false);
        assert_eq!(
            res,
            RotateResult {
                result: 0b00010100,
                carry_flag: false,
                zero_flag: false,
                negative_flag: false,
            }
        );

        let res = rotate_left(0b10000000, false);
        assert_eq!(
            res,
            RotateResult {
                result: 0b00000000,
                carry_flag: true,
                zero_flag: true,
                negative_flag: false,
            }
        );
    }

    #[test]
    fn rotate_right_test() {
        let res = rotate_right(0b11001010, true);
        assert_eq!(
            res,
            RotateResult {
                result: 0b11100101,
                carry_flag: false,
                zero_flag: false,
                negative_flag: true,
            }
        );

        let res = rotate_right(0b11001010, false);
        assert_eq!(
            res,
            RotateResult {
                result: 0b01100101,
                carry_flag: false,
                zero_flag: false,
                negative_flag: false,
            }
        );

        let res = rotate_right(0b10100101, true);
        assert_eq!(
            res,
            RotateResult {
                result: 0b11010010,
                carry_flag: true,
                zero_flag: false,
                negative_flag: true,
            }
        );

        let res = rotate_right(0b10100101, false);
        assert_eq!(
            res,
            RotateResult {
                result: 0b01010010,
                carry_flag: true,
                zero_flag: false,
                negative_flag: false,
            }
        );

        let res = rotate_right(0b00000001, false);
        assert_eq!(
            res,
            RotateResult {
                result: 0b00000000,
                carry_flag: true,
                zero_flag: true,
                negative_flag: false,
            }
        );

        let res = rotate_right(0b00000000, false);
        assert_eq!(
            res,
            RotateResult {
                result: 0b00000000,
                carry_flag: false,
                zero_flag: true,
                negative_flag: false,
            }
        );
    }

    #[test]
    fn overflowing_add_u8_i8_pos_nocarry() {
        let res = overflowing_add_u8_i8(20, 35);
        assert_eq!(res, (55, false));

        let res = overflowing_add_u8_i8(165, 57);
        assert_eq!(res, (222, false));
    }

    #[test]
    fn overflowing_add_u8_i8_neg_nocarry() {
        let res = overflowing_add_u8_i8(20, -17);
        assert_eq!(res, (3, false));

        let res = overflowing_add_u8_i8(150, -96);
        assert_eq!(res, (54, false));
    }

    #[test]
    fn overflowing_add_u8_i8_pos_carry() {
        let res = overflowing_add_u8_i8(170, 125);
        assert_eq!(res, (39, true));

        let res = overflowing_add_u8_i8(230, 57);
        assert_eq!(res, (31, true));
    }

    #[test]
    fn overflowing_add_u8_i8_neg_carry() {
        let res = overflowing_add_u8_i8(20, -117);
        assert_eq!(res, (159, true));

        let res = overflowing_add_u8_i8(100, -101);
        assert_eq!(res, (255, true));
    }

    #[test]
    fn overflowing_add_u8_i8_result_zero_nocarry() {
        let res = overflowing_add_u8_i8(20, -20);
        assert_eq!(res, (0, false));

        let res = overflowing_add_u8_i8(100, -100);
        assert_eq!(res, (0, false));
    }

    #[test]
    fn overflowing_add_u8_i8_result_zero_carry() {
        let res = overflowing_add_u8_i8(220, 36);
        assert_eq!(res, (0, true));

        let res = overflowing_add_u8_i8(156, 100);
        assert_eq!(res, (0, true));
    }

    #[test]
    fn compare_u8_equal() {
        let res = compare_u8(100, 100);
        assert_eq!(
            res,
            CompareU8Result {
                carry: true,
                zero: true,
                neg: false,
            }
        )
    }

    #[test]
    fn compare_u8_greater_pos() {
        let res = compare_u8(100, 10);
        assert_eq!(
            res,
            CompareU8Result {
                carry: true,
                zero: false,
                neg: false,
            }
        )
    }

    #[test]
    fn compare_u8_greater_neg() {
        let res = compare_u8(200, 15);
        assert_eq!(
            res,
            CompareU8Result {
                carry: true,
                zero: false,
                neg: true,
            }
        )
    }

    #[test]
    fn compare_u8_lesser_pos() {
        let res = compare_u8(20, 200);
        assert_eq!(
            res,
            CompareU8Result {
                carry: false,
                zero: false,
                neg: false,
            }
        )
    }

    #[test]
    fn compare_u8_lesser_neg() {
        let res = compare_u8(10, 100);
        assert_eq!(
            res,
            CompareU8Result {
                carry: false,
                zero: false,
                neg: true,
            }
        )
    }

    #[test]
    fn sbc_with_borrow_pos_result() {
        let res = subtract_with_carry(5, 3, true);

        assert_eq!(
            res,
            ADCSBCResult {
                result: 2,
                carry_flag: true,
                overflow_flag: false,
                negative_flag: false,
                zero_flag: false,
            }
        )
    }

    #[test]
    fn sbc_with_borrow_neg_result() {
        let res = subtract_with_carry(5, 6, true);

        assert_eq!(
            res,
            ADCSBCResult {
                result: -1i8 as u8,
                carry_flag: false,
                overflow_flag: false,
                negative_flag: true,
                zero_flag: false,
            }
        )
    }

    #[test]
    fn sbc_all_positive_no_carry() {
        let res = subtract_with_carry(100, 40, false);

        assert_eq!(
            res,
            ADCSBCResult {
                result: 59,
                carry_flag: true,
                overflow_flag: false,
                negative_flag: false,
                zero_flag: false,
            }
        )
    }

    #[test]
    fn sbc_unsigned_overflow() {
        let res = subtract_with_carry(80, 176, true);

        assert_eq!(
            res,
            ADCSBCResult {
                result: 160,
                carry_flag: false,
                overflow_flag: true,
                negative_flag: true,
                zero_flag: false,
            }
        )
    }

    #[test]
    fn adc_with_carry_in() {
        let res = add_with_carry(63, 64, true);
        assert_eq!(
            res,
            ADCSBCResult {
                result: 128,
                carry_flag: false,
                overflow_flag: true,
                negative_flag: true,
                zero_flag: false,
            }
        );
    }

    #[test]
    fn sbc_with_carry_in() {
        let res = subtract_with_carry(-64i8 as u8, 64, false);
        assert_eq!(
            res,
            ADCSBCResult {
                result: 127,
                carry_flag: true,
                overflow_flag: true,
                negative_flag: false,
                zero_flag: false,
            }
        );
    }

    #[test]
    fn adc_all_cases_no_carry() {
        let res = add_with_carry(80, 16, false);
        assert_eq!(
            res,
            ADCSBCResult {
                result: 96,
                carry_flag: false,
                overflow_flag: false,
                negative_flag: false,
                zero_flag: false,
            }
        );

        let res = add_with_carry(80, 80, false);
        assert_eq!(
            res,
            ADCSBCResult {
                result: 160,
                carry_flag: false,
                overflow_flag: true,
                negative_flag: true,
                zero_flag: false,
            }
        );

        let res = add_with_carry(80, 144, false);
        assert_eq!(
            res,
            ADCSBCResult {
                result: 224,
                carry_flag: false,
                overflow_flag: false,
                negative_flag: true,
                zero_flag: false,
            }
        );

        let res = add_with_carry(80, 208, false);
        assert_eq!(
            res,
            ADCSBCResult {
                result: 32,
                carry_flag: true,
                overflow_flag: false,
                negative_flag: false,
                zero_flag: false,
            }
        );

        let res = add_with_carry(208, 16, false);
        assert_eq!(
            res,
            ADCSBCResult {
                result: 224,
                carry_flag: false,
                overflow_flag: false,
                negative_flag: true,
                zero_flag: false,
            }
        );

        let res = add_with_carry(208, 80, false);
        assert_eq!(
            res,
            ADCSBCResult {
                result: 32,
                carry_flag: true,
                overflow_flag: false,
                negative_flag: false,
                zero_flag: false,
            }
        );

        let res = add_with_carry(208, 144, false);
        assert_eq!(
            res,
            ADCSBCResult {
                result: 96,
                carry_flag: true,
                overflow_flag: true,
                negative_flag: false,
                zero_flag: false,
            }
        );

        let res = add_with_carry(208, 208, false);
        assert_eq!(
            res,
            ADCSBCResult {
                result: 160,
                carry_flag: true,
                overflow_flag: false,
                negative_flag: true,
                zero_flag: false,
            }
        );
    }

    #[test]
    fn sbc_all_cases_no_borrow() {
        let res = subtract_with_carry(80, 240, true);
        assert_eq!(
            res,
            ADCSBCResult {
                result: 96,
                carry_flag: false,
                overflow_flag: false,
                negative_flag: false,
                zero_flag: false,
            }
        );

        let res = subtract_with_carry(80, 176, true);
        assert_eq!(
            res,
            ADCSBCResult {
                result: 160,
                carry_flag: false,
                overflow_flag: true,
                negative_flag: true,
                zero_flag: false,
            }
        );

        let res = subtract_with_carry(80, 112, true);
        assert_eq!(
            res,
            ADCSBCResult {
                result: 224,
                carry_flag: false,
                overflow_flag: false,
                negative_flag: true,
                zero_flag: false,
            }
        );

        let res = subtract_with_carry(80, 48, true);
        assert_eq!(
            res,
            ADCSBCResult {
                result: 32,
                carry_flag: true,
                overflow_flag: false,
                negative_flag: false,
                zero_flag: false,
            }
        );

        let res = subtract_with_carry(208, 240, true);
        assert_eq!(
            res,
            ADCSBCResult {
                result: 224,
                carry_flag: false,
                overflow_flag: false,
                negative_flag: true,
                zero_flag: false,
            }
        );

        let res = subtract_with_carry(208, 176, true);
        assert_eq!(
            res,
            ADCSBCResult {
                result: 32,
                carry_flag: true,
                overflow_flag: false,
                negative_flag: false,
                zero_flag: false,
            }
        );

        let res = subtract_with_carry(208, 112, true);
        assert_eq!(
            res,
            ADCSBCResult {
                result: 96,
                carry_flag: true,
                overflow_flag: true,
                negative_flag: false,
                zero_flag: false,
            }
        );

        let res = subtract_with_carry(208, 48, true);
        assert_eq!(
            res,
            ADCSBCResult {
                result: 160,
                carry_flag: true,
                overflow_flag: false,
                negative_flag: true,
                zero_flag: false,
            }
        );
    }

    #[test]
    fn sbc_subrtract_zero() {
        let res = subtract_with_carry(128, 0, false);
        assert_eq!(
            res,
            ADCSBCResult {
                result: 127,
                carry_flag: true,
                overflow_flag: true,
                negative_flag: false,
                zero_flag: false,
            }
        );
    }
}
