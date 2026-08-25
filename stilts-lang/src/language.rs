pub trait Language {
    type Root<'i>: Root;
    type Item<'i>: Item;
    type Block<'i>: Block;
    type Error<'i>;

    fn parse_root<'i>(&self, input: &'i str) -> Result<Self::Root<'i>, Self::Error<'i>>;
}

pub trait Root {
    type ParentId<'a>
    where
        Self: 'a;
    type Item<'a>
    where
        Self: 'a;

    fn parent(&self) -> Option<Self::ParentId<'_>>;
    fn body(&self) -> impl Iterator<Item = Self::Item<'_>>;
}

pub trait Item {
    type Block<'a>
    where
        Self: 'a;
    fn as_block(&self) -> Option<Self::Block<'_>>;
}

pub trait Block {
    type Item<'a>
    where
        Self: 'a;
    type Id<'a>
    where
        Self: 'a;
    fn id(&self) -> Self::Id<'_>;
    fn body(&self) -> impl Iterator<Item = Self::Item<'_>>;
}

pub struct StiltsLanguage {
    delims: stilts_parser::Delims,
}

impl StiltsLanguage {
    pub fn new(delims: stilts_parser::Delims) -> Self {
        Self { delims }
    }
}

impl Language for StiltsLanguage {
    type Root<'i> = stilts_parser::types::Root<'i>;
    type Item<'i> = stilts_parser::types::Item<'i>;
    type Block<'i> = stilts_parser::types::ItemBlock<'i>;
    type Error<'i> = stilts_parser::Error<'i>;

    fn parse_root<'i>(&self, input: &'i str) -> Result<Self::Root<'i>, Self::Error<'i>> {
        stilts_parser::parse_template(input, self.delims.clone())
    }
}

impl<'i> Root for stilts_parser::types::Root<'i> {
    type ParentId<'a>
        = &'a std::borrow::Cow<'i, str>
    where
        Self: 'a;
    type Item<'a>
        = &'a stilts_parser::types::Item<'i>
    where
        Self: 'a;
    fn parent(&self) -> Option<Self::ParentId<'_>> {
        match self.content.first() {
            Some(stilts_parser::types::Item::Expr(stilts_parser::types::Expr::Extends(parent))) => {
                Some(parent)
            }
            _ => None,
        }
    }
    fn body(&self) -> impl Iterator<Item = Self::Item<'_>> {
        self.content.iter()
    }
}

impl<'i> Item for stilts_parser::types::Item<'i> {
    type Block<'a>
        = &'a stilts_parser::types::ItemBlock<'i>
    where
        Self: 'a;
    fn as_block(&self) -> Option<<stilts_parser::types::Item<'i> as Item>::Block<'_>> {
        match self {
            stilts_parser::types::Item::Block(block) => Some(block),
            _ => None,
        }
    }
}

impl<'i> Block for stilts_parser::types::ItemBlock<'i> {
    type Item<'a>
        = &'a stilts_parser::types::Item<'i>
    where
        Self: 'a;
    type Id<'a>
        = &'a std::borrow::Cow<'i, str>
    where
        Self: 'a;

    fn id(&self) -> Self::Id<'_> {
        &self.name
    }

    fn body(&self) -> impl Iterator<Item = Self::Item<'_>> {
        self.content.iter()
    }
}
