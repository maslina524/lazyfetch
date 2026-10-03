pub enum Part<'fp> {
    Text(&'fp str),
    Var(&'fp str),
}

pub struct FormatParserIter<'fp> {
    source: &'fp str,
    rest: &'fp str,
}

impl<'fp> FormatParserIter<'fp> {
    pub const fn new(source: &'fp str) -> Self {
        Self { source, rest: source }
    }
}

impl<'fp> Iterator for FormatParserIter<'fp> {
    type Item = Part<'fp>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rest.is_empty() {
            return None;
        }

        match self.rest.find('{') {
            None => {
                let text = self.rest;
                self.rest = "";
                Some(Part::Text(text))
            }
            Some(0) => {
                let after = &self.rest[1..];
                if let Some(rel) = after.find('}') {
                    let name = &after[..rel];
                    self.rest = &after[rel + 1..];
                    Some(Part::Var(name))
                } else {
                    let text = self.rest;
                    self.rest = "";
                    Some(Part::Text(text))
                }
            }
            Some(pos) => {
                let text = &self.rest[..pos];
                self.rest = &self.rest[pos..];
                Some(Part::Text(text))
            }
        }
    }
}