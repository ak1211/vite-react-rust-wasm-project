// Copyright (c) 2023 Akihiro Yamamoto.
// Licensed under the MIT License <https://spdx.org/licenses/MIT.html>
// See LICENSE file in the project root for full license information.
//
use crate::infrared_remote::{IrCarrierCounter, MarkAndSpaceMicros, Microseconds};
use nom::{
    branch::{alt, permutation},
    bytes::complete::{escaped_transform, take_while_m_n},
    character::complete::{char, digit1, multispace0, none_of},
    combinator::{map, map_res, opt, value},
    error::{convert_error, VerboseError},
    multi::many1,
    sequence::{delimited, tuple},
    Finish, IResult,
};
use std::char::{decode_utf16, REPLACEMENT_CHARACTER};
use std::error::Error;

// 入力文字列を16進数として解釈する。
fn from_hexadecimal_str(s: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(s, 16)
}

// 4桁の16進数(16ビット)
fn four_digits_hexadecimal_lsb_first<'a>(
    s: &'a str,
) -> IResult<&'a str, IrCarrierCounter, nom::error::VerboseError<&'a str>> {
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
fn onoff_pair_mark_and_space<'a>(
    s: &'a str,
) -> IResult<&'a str, MarkAndSpaceMicros, VerboseError<&'a str>> {
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
fn parse_onoff_pair_format<'a>(
    s: &'a str,
) -> IResult<&'a str, Vec<MarkAndSpaceMicros>, VerboseError<&'a str>> {
    many1(delimited(
        multispace0,
        onoff_pair_mark_and_space,
        multispace0,
    ))(s)
}

//
fn json_array_mark_and_space<'a>(
    s: &'a str,
) -> IResult<&'a str, MarkAndSpaceMicros, VerboseError<&'a str>> {
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
fn parse_json_array_format<'a>(
    s: &'a str,
) -> IResult<&'a str, Vec<MarkAndSpaceMicros>, VerboseError<&'a str>> {
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

fn string_literal<'a>(s: &'a str) -> IResult<&'a str, String, VerboseError<&'a str>> {
    delimited(
        char('\"'),
        escaped_transform(
            none_of("\"\\"),
            '\\',
            alt((
                value('\\', char('\\')),
                value('\"', char('\"')),
                value('\'', char('\'')),
                value('\r', char('r')),
                value('\n', char('n')),
                value('\t', char('t')),
                map(
                    permutation((
                        char('u'),
                        take_while_m_n(4, 4, |c: char| c.is_ascii_hexdigit()),
                    )),
                    |(_, code): (char, &str)| -> char {
                        decode_utf16(vec![u16::from_str_radix(code, 16).unwrap()])
                            .nth(0)
                            .unwrap()
                            .unwrap_or(REPLACEMENT_CHARACTER)
                    },
                ),
            )),
        ),
        char('\"'),
    )(s)
}

// pigpioのirrp形式の文字列を解析する
fn parse_pigpio_irrp_format<'a>(
    s: &'a str,
) -> IResult<&'a str, Vec<MarkAndSpaceMicros>, VerboseError<&'a str>> {
    fn json_object<'a>(
        s: &'a str,
    ) -> IResult<&'a str, Vec<MarkAndSpaceMicros>, VerboseError<&'a str>> {
        let (s, _) = multispace0(s)?;
        let (s, _name) = string_literal(s)?;
        let (s, _) = multispace0(s)?;
        let (s, _) = char(':')(s)?;
        let (s, _) = multispace0(s)?;
        let (s, vs) = parse_json_array_format(s)?;
        Ok((s, vs))
    }
    let (s, _) = multispace0(s)?;
    delimited(
        char('{'),
        delimited(multispace0, json_object, multispace0),
        char('}'),
    )(s)
}

// 入力文字列のパーサー
pub fn parse_infrared_code_text<'a>(
    input: &'a str,
) -> Result<Vec<MarkAndSpaceMicros>, Box<dyn Error>> {
    alt((
        parse_onoff_pair_format,
        parse_json_array_format,
        parse_pigpio_irrp_format,
    ))(input)
    .finish()
    .map(|(_, v)| v)
    .map_err(|e| convert_error(input, e).into())
}

#[cfg(test)]
mod parsing_tests {
    use crate::infrared_remote::IrCarrierCounter;
    use crate::parsing::*;
    #[test]
    fn test1_parse_infrared_code_text() {
        let x = parse_infrared_code_text("5601AA00").unwrap();
        let y = vec![MarkAndSpaceMicros::from((
            IrCarrierCounter(0x0156).into(),
            IrCarrierCounter(0x00AA).into(),
        ))];
        assert_eq!(x, y);
    }

    #[test]
    fn test2_parse_infrared_code_text() {
        let x = parse_infrared_code_text("5601AA0017001500").unwrap();
        let y = vec![
            MarkAndSpaceMicros::from((
                IrCarrierCounter(0x0156).into(),
                IrCarrierCounter(0x00AA).into(),
            )),
            MarkAndSpaceMicros::from((
                IrCarrierCounter(0x0017).into(),
                IrCarrierCounter(0x0015).into(),
            )),
        ];
        assert_eq!(x, y);
    }

    #[test]
    fn test3_parse_infrared_code_text() {
        let x = parse_infrared_code_text("5601AA00 17001500").unwrap();
        let y = vec![
            MarkAndSpaceMicros::from((
                IrCarrierCounter(0x0156).into(),
                IrCarrierCounter(0x00AA).into(),
            )),
            MarkAndSpaceMicros::from((
                IrCarrierCounter(0x0017).into(),
                IrCarrierCounter(0x0015).into(),
            )),
        ];
        assert_eq!(x, y);
    }

    #[test]
    fn test4_parse_infrared_code_text() {
        let x = parse_infrared_code_text("5601AA00 17001500").unwrap();
        let y = vec![
            MarkAndSpaceMicros::from((Microseconds(9000), Microseconds(4473))),
            MarkAndSpaceMicros::from((Microseconds(605), Microseconds(552))),
        ];
        assert_eq!(x, y);
    }

    #[test]
    fn test5_parse_infrared_code_text() {
        let x = parse_infrared_code_text("[ 9000, 4473 , 605, 552, ]").unwrap();
        let y = vec![
            MarkAndSpaceMicros::from((Microseconds(9000), Microseconds(4473))),
            MarkAndSpaceMicros::from((Microseconds(605), Microseconds(552))),
        ];
        assert_eq!(x, y);
    }

    #[test]
    fn test6_parse_infrared_code_text() {
        let x = parse_infrared_code_text(r#" { "data": [9000, 4473, 605, 552, ] } "#).unwrap();
        let y = vec![
            MarkAndSpaceMicros::from((Microseconds(9000), Microseconds(4473))),
            MarkAndSpaceMicros::from((Microseconds(605), Microseconds(552))),
        ];
        assert_eq!(x, y);
    }

    #[test]
    fn test7_parse_infrared_code_text() {
        let x = parse_infrared_code_text(r#" { "name" : [ 417 , 448 ] } "#).unwrap();
        let y = vec![MarkAndSpaceMicros::from((
            Microseconds(417),
            Microseconds(448),
        ))];
        assert_eq!(x, y);
    }

    #[test]
    fn test8_parse_infrared_code_text() {
        let x = parse_infrared_code_text(r#"{"name":[417,448,418,]}"#).unwrap();
        let y = vec![
            MarkAndSpaceMicros::from((Microseconds(417), Microseconds(448))),
            MarkAndSpaceMicros::from((Microseconds(418), Microseconds(35000))),
        ];
        assert_eq!(x, y);
    }

    #[test]
    fn test9_parse_infrared_code_text() {
        let x = parse_infrared_code_text(r#" { "name" : [ 417  , 448 , 418 , 450,  ] } "#).unwrap();
        let y = vec![
            MarkAndSpaceMicros::from((Microseconds(417), Microseconds(448))),
            MarkAndSpaceMicros::from((Microseconds(418), Microseconds(450))),
        ];
        assert_eq!(x, y);
    }

    #[test]
    fn test10_parse_infrared_code_text() {
        let x = parse_infrared_code_text(r#" { "name:name" : [ 417  , 448 , 418 , 450,  ] } "#)
            .unwrap();
        let y = vec![
            MarkAndSpaceMicros::from((Microseconds(417), Microseconds(448))),
            MarkAndSpaceMicros::from((Microseconds(418), Microseconds(450))),
        ];
        assert_eq!(x, y);
    }
}
