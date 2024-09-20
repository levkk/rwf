#[derive(Clone, Debug)]
pub enum Part<'a> {
    Slash,
    Identifier(&'a str),
    Segment(&'a str),
}

impl PartialEq for Part<'_> {
    fn eq(&self, other: &Self) -> bool {
        use Part::*;

        match (self, other) {
            (&Identifier(_), &Segment(_)) => true,
            (&Segment(_), &Identifier(_)) => true,
            (&Segment(a), &Segment(b)) => a.eq(b),
            (&Identifier(a), &Identifier(b)) => a.eq(b),
            (&Slash, &Slash) => true,
            _ => false,
        }
    }
}
