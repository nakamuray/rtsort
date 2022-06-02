use nom::{
    character::complete::{one_of, space0},
    combinator::opt,
    number::complete::double,
    sequence::preceded,
    IResult,
};

fn parse_numeric(input: &str) -> IResult<&str, f64> {
    preceded(space0, double)(input)
}

fn unit(input: &str) -> IResult<&str, f64> {
    let (input, unit) = one_of("KMGTkmgt")(input)?;
    let u = match unit.to_ascii_uppercase() {
        'K' => 1000.0,
        'M' => 1000.0 * 1000.0,
        'G' => 1000.0 * 1000.0 * 1000.0,
        'T' => 1000.0 * 1000.0 * 1000.0 * 1000.0,
        _ => unimplemented!(),
    };
    Ok((input, u))
}

fn parse_human_numeric(input: &str) -> IResult<&str, f64> {
    let (input, mut f) = parse_numeric(input)?;
    let (input, u) = opt(unit)(input)?;
    if let Some(u) = u {
        f *= u;
    }

    Ok((input, f))
}

pub fn numeric(input: &str) -> f64 {
    if let Ok((_, f)) = parse_numeric(input) {
        return f;
    } else {
        return 0.0;
    }
}

pub fn human_numeric(input: &str) -> f64 {
    if let Ok((_, f)) = parse_human_numeric(input) {
        return f;
    } else {
        return 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_numeric_test() {
        assert_eq!(parse_numeric("42"), Ok(("", 42.0)));
        assert_eq!(parse_numeric("42.0"), Ok(("", 42.0)));
        assert_eq!(parse_numeric("  42"), Ok(("", 42.0)));
        assert!(parse_numeric("x").is_err());
    }

    #[test]
    fn parse_human_numeric_test() {
        assert_eq!(parse_human_numeric("42"), Ok(("", 42.0)));
        assert_eq!(parse_human_numeric("42K"), Ok(("", 42000.0)));
        assert_eq!(parse_human_numeric("42M"), Ok(("", 42000000.0)));
        assert_eq!(parse_human_numeric("42XXX"), Ok(("XXX", 42.0)));
        assert_eq!(parse_human_numeric("42kXXX"), Ok(("XXX", 42000.0)));
    }
}
