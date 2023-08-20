// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
use crate::infrared_remote::{IrCarrierCounter, MarkAndSpaceMicros, Microseconds};
use nom::{
    branch::alt,
    bytes::complete::{escaped, take_while_m_n},
    character::complete::{alphanumeric1 as alphanumeric, char, digit1, multispace0, one_of},
    combinator::{map_res, opt},
    error::{convert_error, ParseError, VerboseError},
    multi::many1,
    sequence::{delimited, tuple},
    Finish, IResult,
};

// 入力文字列を16進数として解釈する。
fn from_hexadecimal_str(s: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(s, 16)
}

// 4桁の16進数(16ビット)
fn four_digits_hexadecimal_lsb_first(
    s: &str,
) -> IResult<&str, IrCarrierCounter, VerboseError<&str>> {
    // 2桁の16進数(8ビット)
    fn two_digits_hexadecimal(input: &str) -> IResult<&str, u8, VerboseError<&str>> {
        let hexadecimal_8bits_str = take_while_m_n(2, 2, |c: char| c.is_digit(16));
        map_res(hexadecimal_8bits_str, from_hexadecimal_str)(input)
    }
    let (s, (lower, higher)) = tuple((two_digits_hexadecimal, two_digits_hexadecimal))(s)?;
    // 入力値は 下位8ビット -> 上位8ビット の順番なので普通の数字の書き方(高位が前, 下位が後)に入れ替える。
    let value = (higher as u16) << 8 | lower as u16;
    Ok((s, IrCarrierCounter(value)))
}

//
fn onoff_pair_mark_and_space(s: &str) -> IResult<&str, MarkAndSpaceMicros, VerboseError<&str>> {
    tuple((
        four_digits_hexadecimal_lsb_first,
        four_digits_hexadecimal_lsb_first,
    ))(s)
    .map(|(s, ms)| {
        (
            s,
            MarkAndSpaceMicros {
                mark: ms.0.into(),
                space: ms.1.into(),
            },
        )
    })
}

// ONOFFペアの16進数形式の文字列を解析する
fn parse_onoff_pair_format(s: &str) -> IResult<&str, Vec<MarkAndSpaceMicros>, VerboseError<&str>> {
    many1(delimited(
        multispace0,
        onoff_pair_mark_and_space,
        multispace0,
    ))(s)
}

//
fn json_array_mark_and_space(s: &str) -> IResult<&str, MarkAndSpaceMicros, VerboseError<&str>> {
    fn leading(s: &str) -> IResult<&str, u32, VerboseError<&str>> {
        delimited(multispace0, map_res(digit1, str::parse::<u32>), multispace0)(s)
    }
    fn trading(s: &str) -> IResult<&str, u32, VerboseError<&str>> {
        let (s, _) = char(',')(s)?;
        let (s, value) =
            delimited(multispace0, map_res(digit1, str::parse::<u32>), multispace0)(s)?;
        Ok((s, value))
    }
    let (s, value1) = delimited(multispace0, leading, multispace0)(s)?;
    let (s, value2) = opt(delimited(multispace0, trading, multispace0))(s)?;
    let (s, _) = opt(char(','))(s)?;
    Ok((
        s,
        MarkAndSpaceMicros {
            mark: Microseconds(value1),
            //
            // 最後のoffが存在しなかった場合はなにか適当な値を入れる
            //
            space: Microseconds(value2.unwrap_or(35000u32)),
        },
    ))
}

// マイクロ秒ONOFFペアのJSON配列形式の文字列を解析する
fn parse_json_array_format(s: &str) -> IResult<&str, Vec<MarkAndSpaceMicros>, VerboseError<&str>> {
    let (s, _) = multispace0(s)?;
    delimited(
        char('['),
        many1(delimited(
            multispace0,
            json_array_mark_and_space,
            multispace0,
        )),
        char(']'),
    )(s)
}

fn parse_str<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    escaped(alphanumeric, '\\', one_of("\"n\\"))(i)
}

// pigpioのirrp形式の文字列を解析する
fn parse_pigpio_irrp_format(s: &str) -> IResult<&str, Vec<MarkAndSpaceMicros>, VerboseError<&str>> {
    fn json_object(s: &str) -> IResult<&str, (&str, Vec<MarkAndSpaceMicros>), VerboseError<&str>> {
        let (s, _) = multispace0(s)?;
        let (s, name) = delimited(char('"'), parse_str, char('"'))(s)?;
        let (s, _) = multispace0(s)?;
        let (s, _) = char(':')(s)?;
        let (s, _) = multispace0(s)?;
        let (s, vs) = parse_json_array_format(s)?;
        Ok((s, (name, vs)))
    }
    let (s, _) = multispace0(s)?;
    delimited(
        char('{'),
        delimited(multispace0, json_object, multispace0),
        char('}'),
    )(s)
    .map(|(_, vs)| vs)
}

// 入力文字列のパーサー
pub fn parse_infrared_code_text(input: &str) -> Result<Vec<MarkAndSpaceMicros>, String> {
    alt((
        parse_onoff_pair_format,
        parse_json_array_format,
        parse_pigpio_irrp_format,
    ))(input)
    .finish()
    .map(|(_, v)| v)
    .map_err(|e| convert_error(input, e))
}

#[cfg(test)]
mod parsing_tests {
    use crate::infrared_remote::{IrCarrierCounter, MarkAndSpaceIrCarrier};
    use crate::parsing::*;
    #[test]
    fn test1_parse_infrared_code_text() {
        assert_eq!(
            parse_infrared_code_text("5601AA00"),
            Ok(vec![(
                IrCarrierCounter(0x0156).into(),
                IrCarrierCounter(0x00AA).into()
            )
                .into()])
        );
    }

    #[test]
    fn test2_parse_infrared_code_text() {
        assert_eq!(
            parse_infrared_code_text("5601AA00 17001500"),
            Ok(vec![
                (
                    IrCarrierCounter(0x0156).into(),
                    IrCarrierCounter(0x00AA).into()
                )
                    .into(),
                (
                    IrCarrierCounter(0x0017).into(),
                    IrCarrierCounter(0x0015).into()
                )
                    .into(),
            ])
        );
    }

    #[test]
    fn test3_parse_infrared_code_text() {
        assert_eq!(
            parse_infrared_code_text("5601AA00 17001500"),
            Ok(vec![
                MarkAndSpaceIrCarrier {
                    mark: IrCarrierCounter(0x0156),
                    space: IrCarrierCounter(0x00AA),
                }
                .into(),
                MarkAndSpaceIrCarrier {
                    mark: IrCarrierCounter(0x0017),
                    space: IrCarrierCounter(0x0015),
                }
                .into()
            ])
        );
    }

    #[test]
    fn test4_parse_infrared_code_text() {
        assert_eq!(
            parse_infrared_code_text("5601AA00 17001500"),
            Ok(vec![
                (Microseconds(9000), Microseconds(4473)).into(),
                (Microseconds(605), Microseconds(552)).into(),
            ])
        );
    }

    #[test]
    fn test5_parse_infrared_code_text() {
        assert_eq!(
            parse_infrared_code_text("[ 9000, 4473 , 605, 552, ]"),
            Ok(vec![
                (Microseconds(9000), Microseconds(4473)).into(),
                (Microseconds(605), Microseconds(552)).into(),
            ])
        );
    }

    #[test]
    fn test6_parse_infrared_code_text() {
        assert_eq!(
            parse_infrared_code_text(r#" { "data": [9000, 4473, 605, 552, ] } "#),
            Ok(vec![
                (Microseconds(9000), Microseconds(4473)).into(),
                (Microseconds(605), Microseconds(552)).into(),
            ])
        );
    }

    #[test]
    fn test7_parse_infrared_code_text() {
        assert_eq!(
            parse_infrared_code_text(r#" { "name" : [ 417 , 448 ] } "#),
            Ok(vec![(Microseconds(417), Microseconds(448)).into(),])
        );
    }

    #[test]
    fn test8_parse_infrared_code_text() {
        assert_eq!(
            parse_infrared_code_text(r#"{"name":[417,448,418,]}"#),
            Ok(vec![
                (Microseconds(417), Microseconds(448)).into(),
                (Microseconds(418), Microseconds(35000)).into(),
            ])
        );
    }

    #[test]
    fn test9_parse_infrared_code_text() {
        assert_eq!(
            parse_infrared_code_text(r#" { "name" : [ 417  , 448 , 418 , 450,  ] } "#),
            Ok(vec![
                (Microseconds(417), Microseconds(448)).into(),
                (Microseconds(418), Microseconds(450)).into(),
            ])
        );
    }
}
