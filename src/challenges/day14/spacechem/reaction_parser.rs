use super::*;

pub(crate) fn parse_reaction_manifest(s: &str) -> Result<Vec<Reaction>> {
    s.split('\n')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| parse_reaction(s))
        .collect()
}

fn parse_reaction(s: &str) -> Result<Reaction> {
    let mut parts = s.split("=>").filter(|s| !s.is_empty());
    let (inputs, outputs) = parts.next().and_then(|q| parts.next().map(|m| (q, m)))
        .ok_or_else(|| ah!("could not parse reaction from: {}", s))?;
    Ok(
        Reaction {
            inputs: parse_input_list(inputs)?,
            output: parse_quantity(outputs)?,
        }
    )
}

fn parse_input_list(s: &str) -> Result<Vec<Quantity>> {
    s.split(',')
        .filter(|s| !s.is_empty())
        .map(|s| parse_quantity(s))
        .collect()
}

fn parse_quantity(s: &str) -> Result<Quantity> {
    let mut parts = s.split(' ').filter(|s| !s.is_empty());
    let (q_str, m_str) = parts.next().and_then(|q| parts.next().map(|m| (q, m)))
        .ok_or_else(|| ah!("could not parse quantity from: {}", s))?;
    let quantity = parse_str::<i64>(q_str)
        .map_err(|e| ah!("could not parse quantity from: {}: {}", s, e))?;
    Ok(Quantity {
        amount: quantity,
        molecule: Molecule::from(m_str),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_quantity_simple() {
        let expected = Quantity { amount: 32, molecule: Molecule::from("azlp") };
        assert_eq!(parse_quantity("32 azlp").unwrap(), expected);
    }

    #[test]
    fn parse_quantity_extra_space() {
        let expected = Quantity { amount: 0, molecule: Molecule::from("x") };
        assert_eq!(parse_quantity(" 0  x ").unwrap(), expected);
    }

    #[test]
    fn parse_input_list_single() {
        assert_eq!(parse_input_list("32 azlp").unwrap(), vec![
            Quantity { amount: 32, molecule: Molecule::from("azlp") }
        ]);
    }

    #[test]
    fn parse_input_list_long() {
        assert_eq!(parse_input_list(
            "53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV"
        ).unwrap(), vec![
            Quantity { amount: 53, molecule: "STKFG".into() },
            Quantity { amount: 6, molecule: "MNCFX".into() },
            Quantity { amount: 46, molecule: "VJHF".into() },
            Quantity { amount: 81, molecule: "HVMC".into() },
            Quantity { amount: 68, molecule: "CXFTF".into() },
            Quantity { amount: 25, molecule: "GNMV".into() },
        ]);
    }

    #[test]
    fn parse_compact_reaction() {
        assert_eq!(parse_reaction(
            "53 A,6 B,=>1 C"
        ).unwrap(), Reaction {
            inputs: vec![
                Quantity { amount: 53, molecule: "A".into() },
                Quantity { amount: 6, molecule: "B".into() },
            ],
            output: Quantity { amount: 1, molecule: "C".into() },
        }
        );
    }

    #[test]
    fn parse_reaction_complex() {
        assert_eq!(parse_reaction(
            "53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL"
        ).unwrap(), Reaction {
            inputs: vec![
                Quantity { amount: 53, molecule: "STKFG".into() },
                Quantity { amount: 6, molecule: "MNCFX".into() },
                Quantity { amount: 46, molecule: "VJHF".into() },
                Quantity { amount: 81, molecule: "HVMC".into() },
                Quantity { amount: 68, molecule: "CXFTF".into() },
                Quantity { amount: 25, molecule: "GNMV".into() },
            ],
            output: Quantity { amount: 1, molecule: "FUEL".into() },
        }
        );
    }

    #[test]
    fn parse_manifest() {
        assert_eq!(parse_reaction_manifest(
            "53 A,6 B,=>1 C\n4 C, 1 A => 1 CA"
        ).unwrap(), vec![
            Reaction {
                inputs: vec![
                    Quantity { amount: 53, molecule: "A".into() },
                    Quantity { amount: 6, molecule: "B".into() },
                ],
                output: Quantity { amount: 1, molecule: "C".into() },
            },
            Reaction {
                inputs: vec![
                    Quantity { amount: 4, molecule: "C".into() },
                    Quantity { amount: 1, molecule: "A".into() },
                ],
                output: Quantity { amount: 1, molecule: "CA".into() },
            },
        ]
        );
    }
}
