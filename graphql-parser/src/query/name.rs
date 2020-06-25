use super::*;
use crate::Name;

impl<'a> Name<'a> for Definition<'a> {
    fn name(&self) -> Option<&'a str> {
        match self {
            Definition::Operation(o) => o.name,
            Definition::Fragment(f) => Some(f.name),
            Definition::SelectionSet(_) => None,
        }
    }
}

impl<'a> Name<'a> for Selection<'a> {
    fn name(&self) -> Option<&'a str> {
        match self {
            Selection::Field(field) => field.name(),
            Selection::FragmentSpread(spread) => spread.name(),
            Selection::InlineFragment(frag) => frag.name(),
        }
    }
}

impl<'a> Name<'a> for InlineFragment<'a> {
    fn name(&self) -> Option<&'a str> {
        self.type_condition
    }
}
