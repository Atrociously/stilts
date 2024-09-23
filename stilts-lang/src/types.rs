use std::borrow::Cow;

/// The root of a template
///
/// It contains all the content of the template
#[cfg_attr(any(test, feature = "extra-traits"), derive(Clone, Debug, PartialEq, Eq, Hash))]
pub struct Root<'i> {
    pub content: Vec<Item<'i>>,
}

/// Items are the core unit of a template
///
/// A template is made up of a series of *items* which define the structure
/// of the template. An item is either template content, multi-expression block, or a single
/// expression
#[cfg_attr(any(test, feature = "extra-traits"), derive(Clone, Debug, PartialEq, Eq, Hash))]
pub enum Item<'i> {
    Content(Cow<'i, str>),
    Block(ItemBlock<'i>),
    For(ItemFor<'i>),
    If(ItemIf<'i>),
    Match(ItemMatch<'i>),
    Macro(ItemMacro<'i>),
    Expr(Expr<'i>),
}

#[cfg_attr(any(test, feature = "extra-traits"), derive(Clone, Debug, PartialEq, Eq, Hash))]
pub struct ItemBlock<'i> {
    pub name: Cow<'i, str>,
    pub content: Vec<Item<'i>>,
}

#[cfg_attr(any(test, feature = "extra-traits"), derive(Clone, Debug, PartialEq, Eq, Hash))]
pub struct ItemFor<'i> {
    pub label: Option<syn::Label>,
    pub pat: syn::Pat,
    pub expr: syn::Expr,
    pub content: Vec<Item<'i>>,
}

#[cfg_attr(any(test, feature = "extra-traits"), derive(Clone, Debug, PartialEq, Eq, Hash))]
pub struct ItemIf<'i> {
    pub cond: syn::Expr,
    pub content: Vec<Item<'i>>,
    pub branch: IfBranch<'i>,
}

#[cfg_attr(any(test, feature = "extra-traits"), derive(Clone, Debug, PartialEq, Eq, Hash))]
pub enum IfBranch<'i> {
    ElseIf {
        cond: syn::Expr,
        content: Vec<Item<'i>>,
        branch: Box<IfBranch<'i>>,
    },
    Else {
        content: Vec<Item<'i>>,
    },
    End,
}

#[cfg_attr(any(test, feature = "extra-traits"), derive(Clone, Debug, PartialEq, Eq, Hash))]
pub struct ItemMatch<'i> {
    pub expr: syn::Expr,
    pub arms: Vec<MatchArm<'i>>,
}

#[cfg_attr(any(test, feature = "extra-traits"), derive(Clone, Debug, PartialEq, Eq, Hash))]
pub struct MatchArm<'i> {
    pub pat: syn::Pat,
    pub guard: Option<syn::Expr>,
    pub content: Vec<Item<'i>>,
}

#[cfg_attr(any(test, feature = "extra-traits"), derive(Clone, Debug, PartialEq, Eq, Hash))]
pub struct ItemMacro<'i> {
    pub name: syn::Ident,
    pub args: syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    pub content: Vec<Item<'i>>,
}

#[cfg_attr(any(test, feature = "extra-traits"), derive(Clone, Debug, PartialEq, Eq, Hash))]
pub enum Expr<'i> {
    Extends(Cow<'i, str>),
    Include(Cow<'i, str>),
    SuperCall,
    MacroCall {
        name: syn::Ident,
        args: syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>,
    },
    Stmt(syn::Stmt),
    Expr(syn::Expr),
}

macro_rules! impl_into_owned {
    (@enum $name:ident {$($variant:ident ($v:ident) => $do:expr),* $(;$($var2:ident)|+ ($v2:ident) => $do2:expr)?}) => {
        impl $name<'_> {
            pub fn into_owned(self) -> $name<'static> { match self { $(Self::$variant ($v) => $name::$variant($do),)* $($(Self::$var2 ($v2) => $name::$var2($do2),)+)? }}
        }
    };
    (@struct $name:ident $self:ident {$($field:ident: $do:expr),* $(,)?}) => {
        impl $name<'_> {
            #[allow(clippy::all)] pub fn into_owned($self) -> $name<'static> {$name {$($field: $do,)*..$self}}
        }
    };
}

impl_into_owned!(@struct Root self {
    content: self.content.into_iter().map(Item::into_owned).collect(),
});
impl_into_owned!(@enum Item {
    Content(v) => v.into_owned().into();
    Block | For | If | Match | Macro | Expr (v) => v.into_owned()
});
impl_into_owned!(@struct ItemBlock self {
    name: self.name.into_owned().into(),
    content: self.content.into_iter().map(Item::into_owned).collect(),
});
impl_into_owned!(@struct ItemFor self {
    content: self.content.into_iter().map(Item::into_owned).collect()
});
impl_into_owned!(@struct ItemIf self {
    content: self.content.into_iter().map(Item::into_owned).collect(),
    branch: self.branch.into_owned(),
});
impl_into_owned!(@struct ItemMatch self {
    arms: self.arms.into_iter().map(MatchArm::into_owned).collect()
});
impl_into_owned!(@struct MatchArm self {
    content: self.content.into_iter().map(Item::into_owned).collect()
});
impl_into_owned!(@struct ItemMacro self {
    content: self.content.into_iter().map(Item::into_owned).collect()
});

impl IfBranch<'_> {
    pub fn into_owned(self) -> IfBranch<'static> {
        match self {
            Self::ElseIf {
                cond,
                content,
                branch,
            } => IfBranch::ElseIf {
                cond,
                content: content.into_iter().map(Item::into_owned).collect(),
                branch: Box::new(branch.into_owned()),
            },
            Self::Else { content } => IfBranch::Else {
                content: content.into_iter().map(Item::into_owned).collect(),
            },
            Self::End => IfBranch::End,
        }
    }
}

impl<'i> Expr<'i> {
    pub fn into_owned(self) -> Expr<'static> {
        match self {
            Self::Extends(v) => Expr::Extends(v.into_owned().into()),
            Self::Include(v) => Expr::Include(v.into_owned().into()),
            Self::SuperCall => Expr::SuperCall,
            Self::MacroCall { name, args } => Expr::MacroCall { name, args },
            Self::Stmt(v) => Expr::Stmt(v),
            Self::Expr(v) => Expr::Expr(v),
        }
    }
}

pub(crate) struct ForExpr {
    pub label: Option<syn::Label>,
    _for: syn::Token![for],
    pub pat: syn::Pat,
    _in: syn::Token![in],
    pub expr: syn::Expr,
}

pub(crate) struct MatchArmExpr {
    pub pat: syn::Pat,
    pub guard: Option<(syn::Token![if], syn::Expr)>,
}

pub(crate) struct MacroExpr {
    pub name: syn::Ident,
    pub args: syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
}

pub(crate) struct MacroCallExpr {
    pub name: syn::Ident,
    pub args: syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>,
}

impl syn::parse::Parse for ForExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            label: input.parse()?,
            _for: input.parse()?,
            pat: input.call(syn::Pat::parse_single)?,
            _in: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl syn::parse::Parse for MatchArmExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            pat: input.call(syn::Pat::parse_multi)?,
            guard: if input.peek(syn::Token![if]) {
                Some((input.parse()?, input.parse()?))
            } else {
                None
            },
        })
    }
}

impl syn::parse::Parse for MacroExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let content;
        syn::parenthesized!(content in input);
        let args = syn::punctuated::Punctuated::parse_terminated(&content)?;
        Ok(Self { name, args })
    }
}

impl syn::parse::Parse for MacroCallExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let content;
        syn::parenthesized!(content in input);
        let args = syn::punctuated::Punctuated::parse_terminated(&content)?;
        Ok(Self { name, args })
    }
}
