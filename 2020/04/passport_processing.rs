#!/usr/bin/env rustr

// 7:11AM
// 7:32AM
// 8:23AM

#[allow(unused_imports)]
use aoc::{friends::*, AdventOfCode};

use std::convert::{TryFrom, TryInto};
use std::marker::PhantomData;
use std::ops::RangeInclusive;
use std::str::FromStr;

trait Bounds<T> {
    const RANGE: RangeInclusive<T>;
}

#[derive(Debug, PartialEq, Eq)]
struct Bounded<T: FromStr + PartialOrd<T>, R: Bounds<T>>(T, PhantomData<R>);

#[derive(Debug, PartialEq, Eq, Clone)]
enum BoundedFromStrError<E> {
    OutOfRange,
    Other(E),
}

impl<E> From<E> for BoundedFromStrError<E> {
    fn from(e: E) -> Self {
        BoundedFromStrError::Other(e)
    }
}

impl<T: FromStr + PartialOrd<T>, R: Bounds<T>> FromStr for Bounded<T, R> {
    type Err = BoundedFromStrError<<T as FromStr>::Err>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = s.parse()?;

        if R::RANGE.contains(&inner) {
            Ok(Bounded::<_, _>(inner, PhantomData))
        } else {
            Err(BoundedFromStrError::OutOfRange)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Height {
    Centimeters(u16),
    Inches(u16),
}

impl FromStr for Height {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        let h = if let Some(inches) = s.strip_suffix("in") {
            Height::Inches(inches.parse().map_err(|_| ())?)
        } else if let Some(cm) = s.strip_suffix("cm") {
            Height::Centimeters(cm.parse().map_err(|_| ())?)
        } else {
            return Err(());
        };

        if h.valid() {
            Ok(h)
        } else {
            Err(())
        }
    }
}

impl Height {
    fn valid(&self) -> bool {
        match self {
            Height::Centimeters(cm) => (150..=193).contains(cm),
            Height::Inches(inches) => (59..=76).contains(inches),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct HairColor(u32);

impl FromStr for HairColor {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        if let Some(hex_string) = s.strip_prefix("#") {
            Ok(HairColor(
                u32::from_str_radix(hex_string, 16).map_err(|_| ())?,
            ))
        } else {
            Err(())
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum EyeColor {
    Amber,
    Blue,
    Brown,
    Grey,
    Green,
    Hazel,
    Other,
}

impl FromStr for EyeColor {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        use EyeColor::*;
        Ok(match s {
            "amb" => Amber,
            "blu" => Blue,
            "brn" => Brown,
            "gry" => Grey,
            "grn" => Green,
            "hzl" => Hazel,
            "oth" => Other,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct PassportId(u32);

impl FromStr for PassportId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        if s.len() != 9 {
            return Err(());
        }

        Ok(PassportId(s.parse().map_err(|_| ())?))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct DontCare<'a>(PhantomData<&'a str>);

impl FromStr for DontCare<'_> {
    type Err = ();

    fn from_str(_s: &str) -> Result<Self, ()> {
        Ok(DontCare(PhantomData))
    }
}

macro_rules! bound {
    ($ident:ident<$ty:ty> = $range:expr) => {
        #[derive(Debug, PartialEq, Eq, Copy, Clone)]
        struct $ident;

        impl Bounds<$ty> for $ident {
            const RANGE: RangeInclusive<$ty> = $range;
        }
    };
}

bound! { BirthYear<u16> = 1920..=2002 }
bound! { IssueYear<u16> = 2010..=2020 }
bound! { ExpirationYear<u16> = 2020..=2030 }

#[derive(Debug, Default, PartialEq, Eq)]
struct Passport<'a> {
    byr: Option<Option<Bounded<u16, BirthYear>>>,
    iyr: Option<Option<Bounded<u16, IssueYear>>>,
    eyr: Option<Option<Bounded<u16, ExpirationYear>>>,
    hgt: Option<Option<Height>>,
    hcl: Option<Option<HairColor>>,
    ecl: Option<Option<EyeColor>>,
    pid: Option<Option<PassportId>>,
    cid: Option<Option<DontCare<'a>>>,
}

impl<'a> TryFrom<&'a str> for Passport<'a> {
    type Error = ();

    fn try_from(s: &'a str) -> Result<Self, ()> {
        let mut r = Self::default();

        for field in s.split_whitespace() {
            let mut iter = field.split(":");
            let nom = iter.next().ok_or(())?;
            let val = iter.next().ok_or(())?;
            assert_eq!(iter.next(), None);

            macro_rules! fields {
                ($($field:ident)*) => {
                    match nom {
                        $(
                            stringify!($field) => {
                                assert!(r.$field.is_none());
                                r.$field = Some(val.parse().ok());
                            },
                        )*
                        _ => return Err(())
                    }
                };
            }

            fields! { byr iyr eyr hgt hcl ecl pid cid }
        }

        Ok(r)
    }
}

impl Passport<'_> {
    fn has_required_fields(&self) -> bool {
        macro_rules! fields {
            ($first:ident $($field:ident)*) => {
                self.$first.as_ref()
                    $(.and(self.$field.as_ref()))*
            };
        }

        fields!(byr iyr eyr hgt hcl ecl pid).is_some()
    }

    fn is_valid(&self) -> bool {
        if !self.has_required_fields() {
            return false;
        }

        macro_rules! fields {
            ($first:ident $($field:ident)*) => {
                self.$first.as_ref().unwrap().as_ref()
                    $(.and(self.$field.as_ref().unwrap().as_ref()))*
            };
        }

        fields!(byr iyr eyr hgt hcl ecl pid).is_some()
    }
}

fn main() {
    let mut aoc = AdventOfCode::new(2020, 04);
    let input: String = aoc.get_input();

    let passports = input
        .split("\n\n")
        .map(|b| TryInto::<Passport<'_>>::try_into(b).unwrap());

    let p1 = passports
        .clone()
        .filter(|p| p.has_required_fields())
        .count();
    let _ = aoc.submit_p1(p1);

    let p2 = passports.filter(|p| p.is_valid()).count();
    let _ = aoc.submit_p2(p2);
}

#[cfg(test)]
mod tests {
    use super::*;

    const INVALID: &'static str = r#"
eyr:1972 cid:100
hcl:#18171d ecl:amb hgt:170 pid:186cm iyr:2018 byr:1926

iyr:2019
hcl:#602927 eyr:1967 hgt:170cm
ecl:grn pid:012533040 byr:1946

hcl:dab227 iyr:2012
ecl:brn hgt:182cm pid:021572410 eyr:2020 byr:1992 cid:277

hgt:59cm ecl:zzz
eyr:2038 hcl:74454a iyr:2023
pid:3556412378 byr:2007
"#;

    fn parse(s: &str) -> impl Iterator<Item = Passport<'_>> {
        s.split("\n\n")
            .map(|b| TryInto::<Passport<'_>>::try_into(b).unwrap())
    }

    #[test]
    fn invalid() {
        assert_eq!(parse(INVALID).filter(|p| p.is_valid()).count(), 0);
    }

    const VALID: &'static str = r#"
pid:087499704 hgt:74in ecl:grn iyr:2012 eyr:2030 byr:1980
hcl:#623a2f

eyr:2029 ecl:blu cid:129 byr:1989
iyr:2014 pid:896056539 hcl:#a97842 hgt:165cm

hcl:#888785
hgt:164cm byr:2001 iyr:2015 cid:88
pid:545766238 ecl:hzl
eyr:2022

iyr:2010 hgt:158cm hcl:#b6652a ecl:blu byr:1944 eyr:2021 pid:093154719
"#;

    #[test]
    fn valid() {
        assert_eq!(parse(VALID).filter(|p| p.is_valid()).count(), 4);
    }
}
