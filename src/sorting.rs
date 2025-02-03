use std::fmt::Display;

pub enum Sorting {
    Name,
    Size,
    Commits,
    CreationDate,
    ModificationDate,
    Loc,
}

impl Sorting {
    pub fn next(&self) -> Self {
        match *self {
            Self::Name => Self::Size,
            Self::Size => Self::Commits,
            Self::Commits => Self::CreationDate,
            Self::CreationDate => Self::ModificationDate,
            Self::ModificationDate => Self::Loc,
            Self::Loc => Self::Name,
        }
    }

    pub fn previous(&self) -> Self {
        match *self {
            Self::Loc => Self::ModificationDate,
            Self::ModificationDate => Self::CreationDate,
            Self::CreationDate => Self::Commits,
            Self::Commits => Self::Size,
            Self::Size => Self::Name,
            Self::Name => Self::Loc,
        }
    }
}

impl Display for Sorting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sorting::Name => write!(f, "Name"),
            Sorting::Size => write!(f, "Size"),
            Sorting::Commits => write!(f, "Commits"),
            Sorting::CreationDate => write!(f, "Creation Date"),
            Sorting::ModificationDate => write!(f, "Modification Date"),
            Sorting::Loc => write!(f, "Lines of Code"),
        }
    }
}

pub enum Filter {
    All,
    Owned,
    NotOwned,
}

impl Filter {
    pub fn next(&self) -> Self {
        match self {
            Self::All => Self::Owned,
            Self::Owned => Self::NotOwned,
            Self::NotOwned => Self::All,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Self::All => Self::NotOwned,
            Self::Owned => Self::All,
            Self::NotOwned => Self::Owned,
        }
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Filter::All => write!(f, "All"),
            Filter::Owned => write!(f, "Owned"),
            Filter::NotOwned => write!(f, "Not Owned"),
        }
    }
}
